# mdbook-summarizer

[中文](README_CN.md)

Generate an mdBook `SUMMARY.md` from a source tree.

This is a Rust CLI port of the original `generate_summary.py` workflow. It
scans a directory of Markdown files and produces a correctly structured
`SUMMARY.md` with natural numeric sorting, automatic title extraction, and
proper nesting.

## Installation

```sh
cargo install mdbook-summarizer
```

Or build from source:

```sh
git clone https://github.com/wustites/mdbook-summarizer
cd mdbook-summarizer
cargo build --release
```

## Usage

```sh
# Scan src/ and write src/SUMMARY.md (defaults)
mdbook-summarizer

# Specify a custom source directory
mdbook-summarizer --src docs

# Specify a custom output path
mdbook-summarizer --src src --output SUMMARY.md

# Preview generated content without writing
mdbook-summarizer --dry-run

# Verify output is up to date (useful in CI)
mdbook-summarizer --check
```

## Options

| Flag | Description |
|------|-------------|
| `--src <DIR>` | Source directory to scan (default: `src`) |
| `-o, --output <FILE>` | Output file path (default: `<src>/SUMMARY.md`) |
| `--dry-run` | Print generated content to stdout without writing |
| `--check` | Exit with error if output file is outdated |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Behavior

- Treats `README.md`, `index.md`, or `SUMMARY.md` as the directory's index entry.
- Extracts the first level-one Markdown heading (`# Title`) as the entry title.
- Falls back to a title-cased file stem when no heading is found.
- Strips inline code and Markdown links from headings.
- Skips `SUMMARY.md`, dot directories (`.git`, `.github`, etc.), and common
  temporary/backup files (`*.bak`, `*.tmp`, `*~`, files with `backup`, `old`,
  `draft` in the name).
- Sorts directories before files, with natural numeric ordering
  (`chapter2` before `chapter10`).

## CI Integration

Use `--check` in your CI pipeline to ensure `SUMMARY.md` stays in sync:

```sh
mdbook-summarizer --check || (echo "SUMMARY.md is outdated" && exit 1)
```

## License

MIT
