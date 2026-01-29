use ansi_to_html::convert;
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

    for line in ansi_output.lines() {
        // In inline mode with line numbers, delta uses │ before the content
        // Format: "  1 ⋮  2 │content" where ⋮ separates old/new line numbers
        if let Some(pipe_pos) = line.rfind('│') {
            let line_num_part = &line[..pipe_pos];
            let content_part = &line[pipe_pos + '│'.len_utf8()..];

            let line_num_html = convert(line_num_part).unwrap_or_else(|_| html_escape(line_num_part));
            let content_html = convert(content_part).unwrap_or_else(|_| html_escape(content_part));

            // Add newline at end for proper copying
            lines.push(format!(
                "<div class=\"diff-line\"><span class=\"line-num\">{}</span><span class=\"line-content\">{}\n</span></div>",
                line_num_html,
                content_html
            ));
        } else {
            // No │ found, treat entire line as content (headers, separators, etc.)
            let html = convert(line).unwrap_or_else(|_| html_escape(line));
            lines.push(format!("<div class=\"diff-line\"><span class=\"line-content\">{}\n</span></div>", html));
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

/// Split delta's side-by-side ANSI output into left and right panels
fn split_side_by_side_output(ansi_output: &str) -> Result<(String, String), DeltaError> {
    let mut left_lines: Vec<String> = Vec::new();
    let mut right_lines: Vec<String> = Vec::new();

    for line in ansi_output.lines() {
        // Delta uses │ (box drawing character) as the separator between left and right
        // Find the middle separator - it's typically at the midpoint
        if let Some((left, right)) = split_at_middle_separator(line) {
            // Further split each side into line number and content at │
            let left_structured = split_line_number_and_content(&left);
            let right_structured = split_line_number_and_content(&right);
            left_lines.push(left_structured);
            right_lines.push(right_structured);
        } else {
            // No separator found, put entire line in both panels
            let html = convert(line).unwrap_or_else(|_| html_escape(line));
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

        // Convert ANSI to HTML for both parts
        let line_num_html = convert(line_num_part).unwrap_or_else(|_| html_escape(line_num_part));
        let content_html = convert(content_part).unwrap_or_else(|_| html_escape(content_part));

        // Trim trailing whitespace from content
        let content_trimmed = trim_html_trailing_whitespace(&content_html);

        // Only add newline if this is a real line (has line number), not a placeholder
        // Real empty lines have a line number but empty content - they should still get newline
        let newline = if has_line_number { "\n" } else { "" };

        format!(
            "<div class=\"diff-line\"><span class=\"line-num\">{}</span><span class=\"line-content\">{}{}</span></div>",
            line_num_html.replace('│', " "),  // Clean up any remaining │ in line number area
            content_trimmed,
            newline
        )
    } else {
        // No │ found, treat entire line as content
        let html = convert(line).unwrap_or_else(|_| html_escape(line));
        let trimmed = trim_html_trailing_whitespace(&html);
        format!("<div class=\"diff-line\"><span class=\"line-content\">{}\n</span></div>", trimmed)
    }
}

/// Trim trailing whitespace from HTML, handling spans that might contain only whitespace
fn trim_html_trailing_whitespace(html: &str) -> String {
    // First, trim obvious trailing whitespace
    let trimmed = html.trim_end();

    // Remove trailing </span> tags that follow whitespace, then trim again
    let mut result = trimmed.to_string();
    loop {
        let before = result.clone();
        // Remove trailing empty spans or spans with only whitespace
        if result.ends_with("</span>") {
            if let Some(open_pos) = result.rfind("<span") {
                let span_content = &result[open_pos..];
                // Check if the span content (after the >) is just whitespace
                if let Some(gt_pos) = span_content.find('>') {
                    let inner = &span_content[gt_pos + 1..span_content.len() - 7]; // 7 = "</span>".len()
                    if inner.trim().is_empty() {
                        result = result[..open_pos].to_string();
                        result = result.trim_end().to_string();
                        continue;
                    }
                }
            }
        }
        if before == result {
            break;
        }
    }

    result.trim_end().to_string()
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

pub fn get_file_content(path: &Path) -> Result<String, DeltaError> {
    Ok(std::fs::read_to_string(path)?)
}
