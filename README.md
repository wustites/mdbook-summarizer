# mdbook-summarizer

[中文文档](README_CN.md)

Generate an mdBook `SUMMARY.md` from a source tree.

This is a Rust CLI port of the original `generate_summary.py` workflow. By
default it scans `src` and writes `src/SUMMARY.md`.

## Installation

```sh
cargo install mdbook-summarizer
```

## Usage

```sh
mdbook-summarizer
mdbook-summarizer --src src
mdbook-summarizer --src src --output src/SUMMARY.md
mdbook-summarizer --auto-readme
mdbook-summarizer --dry-run
mdbook-summarizer --check
```

## Options

| Flag | Description |
|------|-------------|
| `--src <DIR>` | Source directory to scan (default: `src`) |
| `-o, --output <FILE>` | Output file (default: `<src>/SUMMARY.md`) |
| `--auto-readme` | Generate `README.md` (`# dirname`) for dirs without an index |
| `--dry-run` | Print generated content without writing |
| `--check` | Exit with error if output is stale |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Behavior

- Treats `README.md` or `index.md` as a directory entry.
- Extracts the first level-one Markdown heading as the title.
- Falls back to a title-cased file stem when no heading exists.
- `--auto-readme` generates `README.md` (`# dirname`) for dirs without an index.
- Skips generated `SUMMARY.md`, dot directories, and common temporary files.
- Sorts directories before files, with natural numeric ordering.

## CI Integration

Use `--check` in CI to keep `SUMMARY.md` in sync:

```sh
mdbook-summarizer --check || (echo "SUMMARY.md is stale" && exit 1)
```

## License

MIT
