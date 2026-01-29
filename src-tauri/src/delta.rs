use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeltaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ANSI conversion error: {0}")]
    AnsiConversion(String),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Delta not installed")]
    DeltaNotInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffOptions {
    pub side_by_side: bool,
    pub line_numbers: bool,
    pub collapsed: bool,
    pub show_whitespace: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub html: String,
    pub has_changes: bool,
    pub hunk_count: usize,
    /// For custom side-by-side layout - left (old) file HTML
    pub left_html: Option<String>,
    /// For custom side-by-side layout - right (new) file HTML
    pub right_html: Option<String>,
}

pub fn check_delta_installed() -> bool {
    Command::new("delta")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn generate_diff(
    left_path: Option<&Path>,
    right_path: Option<&Path>,
    options: &DiffOptions,
) -> Result<DiffResult, DeltaError> {
    if !check_delta_installed() {
        return Err(DeltaError::DeltaNotInstalled);
    }

    // Handle added/deleted/modified files
    let (left, right) = match (left_path, right_path) {
        (Some(l), Some(r)) => (l, r),
        (None, Some(r)) => {
            // New file - diff against /dev/null
            return generate_diff_with_delta(Path::new("/dev/null"), r, options, true);
        }
        (Some(l), None) => {
            // Deleted file - diff against /dev/null
            return generate_diff_with_delta(l, Path::new("/dev/null"), options, true);
        }
        (None, None) => {
            return Ok(DiffResult {
                html: String::new(),
                has_changes: false,
                hunk_count: 0,
                left_html: None,
                right_html: None,
            });
        }
    };

    generate_diff_with_delta(left, right, options, false)
}

fn generate_diff_with_delta(
    left: &Path,
    right: &Path,
    options: &DiffOptions,
    _is_new_or_deleted: bool,
) -> Result<DiffResult, DeltaError> {
    // Generate unified diff
    let context_lines = if options.collapsed { 3 } else { 99999 };

    let diff_output = Command::new("diff")
        .arg(format!("-U{}", context_lines))
        .arg(left)
        .arg(right)
        .output()?;

    let diff_text = String::from_utf8(diff_output.stdout)?;

    // No changes
    if diff_text.is_empty() && diff_output.status.code() == Some(0) {
        return Ok(DiffResult {
            html: "<div class=\"no-changes\">Files are identical</div>".to_string(),
            has_changes: false,
            hunk_count: 0,
            left_html: None,
            right_html: None,
        });
    }

    let hunk_count = diff_text.lines().filter(|l| l.starts_with("@@")).count();

    // Run through delta
    let mut delta_cmd = Command::new("delta");

    if options.side_by_side {
        delta_cmd.arg("--side-by-side");
        // Use a reasonable width - each side gets half
        delta_cmd.args(["--width", "160"]);
    }

    if options.line_numbers {
        delta_cmd.arg("--line-numbers");
    }

    // Hide file headers (we show them in the UI)
    delta_cmd.args(["--file-style", "omit"]);
    delta_cmd.args(["--hunk-header-style", "omit"]);

    // Use a dark theme
    delta_cmd.args(["--dark"]);

    delta_cmd.stdin(Stdio::piped());
    delta_cmd.stdout(Stdio::piped());
    delta_cmd.stderr(Stdio::piped());

    let mut child = delta_cmd.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(diff_text.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let ansi_output = String::from_utf8(output.stdout)?;

    // For side-by-side mode, split delta's output into left and right panels
    if options.side_by_side {
        let (left_html, right_html) = split_side_by_side_output(&ansi_output)?;
        return Ok(DiffResult {
            html: String::new(),
            has_changes: true,
            hunk_count,
            left_html: Some(left_html),
            right_html: Some(right_html),
        });
    }

    // Inline mode: process each line to separate line numbers from content
    let mut lines: Vec<String> = Vec::new();
    let mut prev_line_num: Option<u32> = None;

    for line in ansi_output.lines() {
        // In inline mode with line numbers, delta uses │ before the content
        // Format: "  1 ⋮  2 │content" where ⋮ separates old/new line numbers
        if let Some(pipe_pos) = line.rfind('│') {
            let line_num_part = &line[..pipe_pos];
            let content_part = &line[pipe_pos + '│'.len_utf8()..];

            // Extract line number to detect gaps
            let curr_line_num = extract_line_number(line_num_part);

            // Check for gaps in line numbers (indicating hidden context)
            if let (Some(prev), Some(curr)) = (prev_line_num, curr_line_num) {
                if curr > prev + 1 {
                    lines.push(create_hunk_separator());
                }
            }

            // Update previous line number
            if curr_line_num.is_some() {
                prev_line_num = curr_line_num;
            }

            // Extract line-level background for continuous highlighting
            let line_bg = extract_line_background(content_part);
            let style = match line_bg {
                Some(bg) => format!(" style='background:{}'", bg),
                None => String::new(),
            };

            let line_num_html = ansi_to_html(line_num_part);
            let content_html = ansi_to_html(content_part);

            // Add newline at end for proper copying
            lines.push(format!(
                "<div class=\"diff-line\"{}><span class=\"line-num\">{}</span><span class=\"line-content\">{}\n</span></div>",
                style,
                line_num_html,
                content_html
            ));
        } else {
            // No │ found, treat entire line as content (headers, separators, etc.)
            let line_bg = extract_line_background(line);
            let style = match line_bg {
                Some(bg) => format!(" style='background:{}'", bg),
                None => String::new(),
            };
            let html = ansi_to_html(line);
            lines.push(format!("<div class=\"diff-line\"{}><span class=\"line-content\">{}\n</span></div>", style, html));
        }
    }

    // Wrap in container div
    let styled_html = format!(
        "<div class=\"delta-output\">{}</div>",
        lines.join("\n")
    );

    Ok(DiffResult {
        html: styled_html,
        has_changes: true,
        hunk_count,
        left_html: None,
        right_html: None,
    })
}

/// Extract line number from the line number part of delta output
fn extract_line_number(line_num_part: &str) -> Option<u32> {
    let visible = strip_ansi_codes(line_num_part);
    // Find the last number in the visible text (handles "  1 " format)
    visible
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .last()
}

/// Create a separator row to indicate hidden lines between hunks
fn create_hunk_separator() -> String {
    "<div class=\"diff-separator\"></div>".to_string()
}

/// Split delta's side-by-side ANSI output into left and right panels
fn split_side_by_side_output(ansi_output: &str) -> Result<(String, String), DeltaError> {
    let mut left_lines: Vec<String> = Vec::new();
    let mut right_lines: Vec<String> = Vec::new();
    let mut prev_left_line_num: Option<u32> = None;
    let mut prev_right_line_num: Option<u32> = None;

    for line in ansi_output.lines() {
        // Delta uses │ (box drawing character) as the separator between left and right
        // Find the middle separator - it's typically at the midpoint
        if let Some((left, right)) = split_at_middle_separator(line) {
            // Extract line numbers to detect gaps
            let left_line_num = if let Some(pipe_pos) = left.rfind('│') {
                extract_line_number(&left[..pipe_pos])
            } else {
                None
            };
            let right_line_num = if let Some(pipe_pos) = right.rfind('│') {
                extract_line_number(&right[..pipe_pos])
            } else {
                None
            };

            // Check for gaps in line numbers (indicating hidden context)
            let left_gap = match (prev_left_line_num, left_line_num) {
                (Some(prev), Some(curr)) => curr > prev + 1,
                _ => false,
            };
            let right_gap = match (prev_right_line_num, right_line_num) {
                (Some(prev), Some(curr)) => curr > prev + 1,
                _ => false,
            };

            // Insert separator if there's a gap on either side
            if left_gap || right_gap {
                left_lines.push(create_hunk_separator());
                right_lines.push(create_hunk_separator());
            }

            // Update previous line numbers
            if left_line_num.is_some() {
                prev_left_line_num = left_line_num;
            }
            if right_line_num.is_some() {
                prev_right_line_num = right_line_num;
            }

            // Further split each side into line number and content at │
            let left_structured = split_line_number_and_content(&left);
            let right_structured = split_line_number_and_content(&right);
            left_lines.push(left_structured);
            right_lines.push(right_structured);
        } else {
            // No separator found, put entire line in both panels
            let html = ansi_to_html(line);
            let trimmed = trim_html_trailing_whitespace(&html);
            left_lines.push(format!("<div class=\"diff-line\"><span class=\"line-content\">{}</span></div>", trimmed));
            right_lines.push(format!("<div class=\"diff-line\"><span class=\"line-content\">{}</span></div>", trimmed));
        }
    }

    let left_html = format!(
        "<div class=\"sbs-panel\">{}</div>",
        left_lines.join("\n")
    );
    let right_html = format!(
        "<div class=\"sbs-panel\">{}</div>",
        right_lines.join("\n")
    );

    Ok((left_html, right_html))
}

/// Extract the first background color from ANSI codes (line-level highlight)
fn extract_line_background(ansi: &str) -> Option<String> {
    let mut in_escape = false;
    let mut escape_buf = String::new();

    for c in ansi.chars() {
        if c == '\x1b' {
            in_escape = true;
            escape_buf.clear();
            escape_buf.push(c);
        } else if in_escape {
            escape_buf.push(c);
            if c == 'm' {
                // Parse the escape sequence for background color
                if escape_buf.len() > 2 {
                    let seq = &escape_buf[2..escape_buf.len() - 1];
                    let parts: Vec<&str> = seq.split(';').collect();
                    let mut i = 0;
                    while i < parts.len() {
                        if parts[i] == "48" && i + 4 < parts.len() && parts[i + 1] == "2" {
                            // RGB background: 48;2;r;g;b
                            let r: u8 = parts[i + 2].parse().unwrap_or(0);
                            let g: u8 = parts[i + 3].parse().unwrap_or(0);
                            let b: u8 = parts[i + 4].parse().unwrap_or(0);
                            return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
                        }
                        i += 1;
                    }
                }
                in_escape = false;
            }
        }
    }
    None
}

/// Split a panel line into line number (non-selectable) and content parts
fn split_line_number_and_content(line: &str) -> String {
    // Line format: "│  1 │content" or "  1 │content" or just "content"
    // Find the last │ which separates line number from content

    if let Some(last_pipe_pos) = line.rfind('│') {
        let line_num_part = &line[..last_pipe_pos];
        let content_part = &line[last_pipe_pos + '│'.len_utf8()..];

        // Check if line number part has actual digits (not a placeholder line)
        let line_num_visible = strip_ansi_codes(line_num_part);
        let has_line_number = line_num_visible.chars().any(|c| c.is_ascii_digit());

        // Extract line-level background color to apply to the whole line
        let line_bg = extract_line_background(content_part);

        // Convert ANSI to HTML for both parts
        let line_num_html = ansi_to_html(line_num_part);
        let content_html = ansi_to_html(content_part);

        // Trim trailing whitespace from content
        let content_trimmed = trim_html_trailing_whitespace(&content_html);

        // Only add newline if this is a real line (has line number), not a placeholder
        // Real empty lines have a line number but empty content - they should still get newline
        let newline = if has_line_number { "\n" } else { "" };

        // Apply line background to the diff-line div for continuous highlighting
        let style = match line_bg {
            Some(bg) => format!(" style='background:{}'", bg),
            None => String::new(),
        };

        format!(
            "<div class=\"diff-line\"{}><span class=\"line-num\">{}</span><span class=\"line-content\">{}{}</span></div>",
            style,
            line_num_html.replace('│', " "),  // Clean up any remaining │ in line number area
            content_trimmed,
            newline
        )
    } else {
        // No │ found, treat entire line as content
        let line_bg = extract_line_background(line);
        let html = ansi_to_html(line);
        let trimmed = trim_html_trailing_whitespace(&html);
        let style = match line_bg {
            Some(bg) => format!(" style='background:{}'", bg),
            None => String::new(),
        };
        format!("<div class=\"diff-line\"{}><span class=\"line-content\">{}\n</span></div>", style, trimmed)
    }
}

/// Trim trailing whitespace from HTML content
/// Just do simple trimming - don't try to manipulate span structure
fn trim_html_trailing_whitespace(html: &str) -> String {
    html.trim_end().to_string()
}

/// Split a line at the middle vertical bar separator
/// Delta's side-by-side output with line numbers has format:
/// │  1 │left_content          │  1 │right_content
fn split_at_middle_separator(line: &str) -> Option<(String, String)> {
    // Delta uses │ (U+2502 BOX DRAWINGS LIGHT VERTICAL) as separator
    // Collect all separator byte positions
    let separators: Vec<usize> = line.match_indices('│').map(|(i, _)| i).collect();

    if separators.is_empty() {
        return None;
    }

    // Calculate the visible length (excluding ANSI escape codes)
    let visible_len = strip_ansi_codes(line).chars().count();
    let target_mid = visible_len / 2;

    // Find the separator closest to the visual middle
    let mut best_sep_idx = 0;
    let mut best_distance = usize::MAX;

    for (idx, &byte_pos) in separators.iter().enumerate() {
        let prefix = &line[..byte_pos];
        let visible_pos = strip_ansi_codes(prefix).chars().count();
        let distance = (visible_pos as isize - target_mid as isize).unsigned_abs();

        if distance < best_distance {
            best_distance = distance;
            best_sep_idx = idx;
        }
    }

    let mid_sep_pos = separators[best_sep_idx];

    // Split at the middle separator
    let left = &line[..mid_sep_pos];
    let right = &line[mid_sep_pos + '│'.len_utf8()..];

    // Don't replace │ here - let split_line_number_and_content handle it
    Some((left.to_string(), right.to_string()))
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert ANSI escape codes to HTML spans
/// Custom implementation to fix word-level highlighting (the ansi-to-html crate has bugs)
fn ansi_to_html(input: &str) -> String {
    let mut result = String::new();
    let mut current_fg: Option<String> = None;
    let mut current_bg: Option<String> = None;
    let mut in_escape = false;
    let mut escape_buf = String::new();

    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            escape_buf.clear();
            escape_buf.push(c);
        } else if in_escape {
            escape_buf.push(c);
            if c == 'm' {
                // Parse the escape sequence
                if escape_buf.len() > 2 {
                    let seq = &escape_buf[2..escape_buf.len() - 1]; // Remove \x1b[ and m
                    let (new_fg, new_bg) = parse_ansi_codes(seq, &current_fg, &current_bg);

                    // If colors changed, close old span and open new
                    if new_bg != current_bg || new_fg != current_fg {
                        if current_bg.is_some() || current_fg.is_some() {
                            result.push_str("</span>");
                        }
                        current_bg = new_bg;
                        current_fg = new_fg;
                        if current_bg.is_some() || current_fg.is_some() {
                            result.push_str("<span style='");
                            if let Some(ref bg) = current_bg {
                                result.push_str(&format!("background:{};", bg));
                            }
                            if let Some(ref fg) = current_fg {
                                result.push_str(&format!("color:{};", fg));
                            }
                            result.push_str("'>");
                        }
                    }
                }
                in_escape = false;
            }
        } else {
            // Escape HTML entities
            match c {
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                '&' => result.push_str("&amp;"),
                '"' => result.push_str("&quot;"),
                _ => result.push(c),
            }
        }
    }

    // Close any remaining span
    if current_bg.is_some() || current_fg.is_some() {
        result.push_str("</span>");
    }

    result
}

/// Parse ANSI SGR codes and return new foreground/background colors
fn parse_ansi_codes(
    seq: &str,
    current_fg: &Option<String>,
    current_bg: &Option<String>,
) -> (Option<String>, Option<String>) {
    let mut fg = current_fg.clone();
    let mut bg = current_bg.clone();
    let parts: Vec<&str> = seq.split(';').collect();
    let mut i = 0;

    while i < parts.len() {
        match parts[i] {
            "0" => {
                // Reset all attributes
                fg = None;
                bg = None;
            }
            "38" => {
                // Foreground color
                if i + 1 < parts.len() && parts[i + 1] == "2" && i + 4 < parts.len() {
                    // RGB color: 38;2;r;g;b
                    let r: u8 = parts[i + 2].parse().unwrap_or(0);
                    let g: u8 = parts[i + 3].parse().unwrap_or(0);
                    let b: u8 = parts[i + 4].parse().unwrap_or(0);
                    fg = Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
                    i += 4;
                } else if i + 1 < parts.len() && parts[i + 1] == "5" && i + 2 < parts.len() {
                    // 256 color: 38;5;n - convert to approximate RGB
                    let n: u8 = parts[i + 2].parse().unwrap_or(0);
                    fg = Some(ansi_256_to_rgb(n));
                    i += 2;
                }
            }
            "48" => {
                // Background color
                if i + 1 < parts.len() && parts[i + 1] == "2" && i + 4 < parts.len() {
                    // RGB color: 48;2;r;g;b
                    let r: u8 = parts[i + 2].parse().unwrap_or(0);
                    let g: u8 = parts[i + 3].parse().unwrap_or(0);
                    let b: u8 = parts[i + 4].parse().unwrap_or(0);
                    bg = Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
                    i += 4;
                } else if i + 1 < parts.len() && parts[i + 1] == "5" && i + 2 < parts.len() {
                    // 256 color: 48;5;n
                    let n: u8 = parts[i + 2].parse().unwrap_or(0);
                    bg = Some(ansi_256_to_rgb(n));
                    i += 2;
                }
            }
            // Basic colors (30-37 foreground, 40-47 background)
            "30" => fg = Some("#000000".to_string()),
            "31" => fg = Some("#aa0000".to_string()),
            "32" => fg = Some("#00aa00".to_string()),
            "33" => fg = Some("#aaaa00".to_string()),
            "34" => fg = Some("#0000aa".to_string()),
            "35" => fg = Some("#aa00aa".to_string()),
            "36" => fg = Some("#00aaaa".to_string()),
            "37" => fg = Some("#aaaaaa".to_string()),
            "40" => bg = Some("#000000".to_string()),
            "41" => bg = Some("#aa0000".to_string()),
            "42" => bg = Some("#00aa00".to_string()),
            "43" => bg = Some("#aaaa00".to_string()),
            "44" => bg = Some("#0000aa".to_string()),
            "45" => bg = Some("#aa00aa".to_string()),
            "46" => bg = Some("#00aaaa".to_string()),
            "47" => bg = Some("#aaaaaa".to_string()),
            // Bright colors (90-97 foreground, 100-107 background)
            "90" => fg = Some("#555555".to_string()),
            "91" => fg = Some("#ff5555".to_string()),
            "92" => fg = Some("#55ff55".to_string()),
            "93" => fg = Some("#ffff55".to_string()),
            "94" => fg = Some("#5555ff".to_string()),
            "95" => fg = Some("#ff55ff".to_string()),
            "96" => fg = Some("#55ffff".to_string()),
            "97" => fg = Some("#ffffff".to_string()),
            _ => {}
        }
        i += 1;
    }

    (fg, bg)
}

/// Convert ANSI 256 color code to RGB hex
fn ansi_256_to_rgb(n: u8) -> String {
    match n {
        0..=15 => {
            // Standard colors
            let colors = [
                "#000000", "#aa0000", "#00aa00", "#aaaa00", "#0000aa", "#aa00aa", "#00aaaa",
                "#aaaaaa", "#555555", "#ff5555", "#55ff55", "#ffff55", "#5555ff", "#ff55ff",
                "#55ffff", "#ffffff",
            ];
            colors[n as usize].to_string()
        }
        16..=231 => {
            // 216 color cube: 6x6x6
            let n = n - 16;
            let r = (n / 36) % 6;
            let g = (n / 6) % 6;
            let b = n % 6;
            let to_val = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            format!("#{:02x}{:02x}{:02x}", to_val(r), to_val(g), to_val(b))
        }
        232..=255 => {
            // Grayscale: 24 shades
            let gray = 8 + (n - 232) * 10;
            format!("#{:02x}{:02x}{:02x}", gray, gray, gray)
        }
    }
}

pub fn get_file_content(path: &Path) -> Result<String, DeltaError> {
    Ok(std::fs::read_to_string(path)?)
}
