# mdbook-summarizer

Generate an mdBook `SUMMARY.md` from a source tree.

This is a Rust CLI port of the original `generate_summary.py` workflow. By
default it scans `src` and writes `src/SUMMARY.md`.

## Usage

```sh
mdbook-summarizer
mdbook-summarizer --src src
mdbook-summarizer --src src --output src/SUMMARY.md
mdbook-summarizer --dry-run
mdbook-summarizer --check
```

## Behavior

- Treats `README.md` or `index.md` as a directory entry.
- Extracts the first level-one Markdown heading as the title.
- Falls back to a title-cased file stem when no heading exists.
- Skips generated `SUMMARY.md`, dot directories, and common temporary files.
- Sorts directories before files, with natural numeric ordering.
