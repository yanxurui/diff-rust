# diff-rust - GUI Diff Tool

A Tauri-based GUI diff tool with delta-quality visuals, designed for `git difftool --dir-diff` integration.

## Features

- **File Tree Panel**: Hierarchical view of changed files with status indicators (M)odified, (A)dded, (D)eleted
- **Syntax Highlighted Diff**: Uses [delta](https://github.com/dandavison/delta) for beautiful, word-level diff highlighting
- **View Modes**: Toggle between inline and side-by-side views
- **Keyboard Navigation**: j/k for file navigation, n/N for hunk navigation
- **Dark Theme**: Modern dark UI optimized for code review

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/)
- [delta](https://github.com/dandavison/delta) (optional but recommended for best diff rendering)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/diff-rust.git
cd diff-rust

# Install dependencies
npm install

# Build for production
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Usage

### Standalone

```bash
diff-rust /path/to/old/directory /path/to/new/directory
```

### Git Integration

Configure as your git difftool:

```bash
git config --global diff.tool diff-rust
git config --global difftool.diff-rust.cmd 'diff-rust "$LOCAL" "$REMOTE"'
```

Then use with:

```bash
git difftool --dir-diff HEAD~1
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` | Next file |
| `k` | Previous file |
| `n` | Next hunk |
| `N` | Previous hunk |

## Development

```bash
# Start development server
npm run tauri dev
```

## Tech Stack

- **Backend**: Rust + Tauri 2.0
- **Frontend**: Vue.js 3 + TypeScript + Tailwind CSS
- **Diff Rendering**: delta (subprocess) + ansi-to-html

## License

MIT
