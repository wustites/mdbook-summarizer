use std::cmp::Ordering;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const INDEX_NAMES: &[&str] = &["README.md", "index.md", "SUMMARY.md"];
const EXCLUDE_DIRS: &[&str] = &[
    ".git",
    ".github",
    ".claude",
    ".qwen",
    "node_modules",
    "__pycache__",
];

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Args {
    src: PathBuf,
    output: Option<PathBuf>,
    dry_run: bool,
    check: bool,
}

#[derive(Debug, Clone)]
enum Entry {
    Dir {
        title: String,
        path: String,
        children: Vec<Entry>,
    },
    File {
        title: String,
        path: String,
    },
}

fn main() -> Result<()> {
    let args = parse_args(env::args().skip(1))?;
    let src_dir = args.src;
    let output = args.output.unwrap_or_else(|| src_dir.join("SUMMARY.md"));

    println!("Scanning directory: {}", src_dir.display());

    if !src_dir.exists() {
        return Err(format!("Error: directory {} does not exist", src_dir.display()).into());
    }

    let entries = collect_files_and_dirs(&src_dir, &src_dir)?;
    let root_index = find_index_file(&src_dir)?;
    let content = generate_summary(&src_dir, &entries, root_index.as_deref())?;
    let total_entries = count_entries(&entries) + usize::from(root_index.is_some());

    if args.dry_run {
        print!("{content}");
        println!("Contains {total_entries} entries");
        return Ok(());
    }

    if args.check {
        let current = fs::read_to_string(&output).unwrap_or_default();
        if current != content {
            return Err(format!("{} needs to be updated", output.display()).into());
        }
        println!("{} is up to date", output.display());
        println!("Contains {total_entries} entries");
        return Ok(());
    }

    match fs::read_to_string(&output) {
        Ok(current) if current == content => {
            println!("{} is unchanged", output.display());
        }
        _ => {
            fs::write(&output, content)?;
            println!("Generated {}", output.display());
        }
    }

    println!("Contains {total_entries} entries");
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args>
where
    I: IntoIterator<Item = String>,
{
    let mut src = PathBuf::from("src");
    let mut output = None;
    let mut dry_run = false;
    let mut check = false;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("mdbook-summarizer {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--src" => {
                src = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--src requires a directory argument".to_string())?,
                );
            }
            "-o" | "--output" => {
                output =
                    Some(PathBuf::from(iter.next().ok_or_else(|| {
                        "--output requires a file argument".to_string()
                    })?));
            }
            "--dry-run" => dry_run = true,
            "--check" => check = true,
            value => return Err(format!("Unknown argument: {value}").into()),
        }
    }

    Ok(Args {
        src,
        output,
        dry_run,
        check,
    })
}

fn print_help() {
    println!(
        "Generate mdBook SUMMARY.md files from a source tree\n\n\
Usage: mdbook-summarizer [OPTIONS]\n\n\
Options:\n  \
--src <DIR>        Source directory to scan [default: src]\n  \
-o, --output <FILE> Output file [default: <src>/SUMMARY.md]\n  \
--dry-run          Print generated content without writing\n  \
--check            Verify the output file is up to date\n  \
-h, --help         Print help\n  \
-V, --version      Print version"
    );
}

fn should_include_file(filename: &str) -> bool {
    let lower_name = filename.to_lowercase();
    if lower_name.ends_with('~')
        || lower_name.ends_with(".bak")
        || lower_name.ends_with(".tmp")
        || lower_name.ends_with(".swp")
        || lower_name.ends_with(".swo")
    {
        return false;
    }

    let stem = lower_name
        .trim_end_matches(".md")
        .split(['.', '_', '-'])
        .collect::<Vec<_>>();
    !stem
        .iter()
        .any(|part| matches!(*part, "backup" | "old" | "temp" | "tmp" | "draft" | "bak"))
}

fn should_include_dir(dirname: &str) -> bool {
    !dirname.starts_with('.') && !EXCLUDE_DIRS.contains(&dirname)
}

fn clean_markdown_title(title: &str) -> String {
    let cleaned = title.trim().trim_matches('#').trim();
    strip_markdown_links(&strip_inline_code(cleaned))
}

fn strip_inline_code(value: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;

    for ch in value.chars() {
        if ch == '`' {
            in_code = !in_code;
            continue;
        }
        output.push(ch);
    }

    output
}

fn strip_markdown_links(value: &str) -> String {
    let mut output = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '[' {
            if let Some(close_bracket) = chars[index + 1..].iter().position(|ch| *ch == ']') {
                let close_bracket = index + 1 + close_bracket;
                if chars.get(close_bracket + 1) == Some(&'(') {
                    if let Some(close_paren) =
                        chars[close_bracket + 2..].iter().position(|ch| *ch == ')')
                    {
                        output.extend(chars[index + 1..close_bracket].iter());
                        index = close_bracket + 3 + close_paren;
                        continue;
                    }
                }
            }
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn extract_title_from_md(filepath: &Path) -> String {
    let content = fs::read_to_string(filepath).unwrap_or_default();
    let content_prefix = content.chars().take(2048).collect::<String>();

    for line in content_prefix.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            return clean_markdown_title(title);
        }
    }

    fallback_title(filepath)
}

fn fallback_title(filepath: &Path) -> String {
    let stem = filepath
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    title_case_ascii(&stem.replace(['_', '-'], " "))
}

fn title_case_ascii(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn find_index_file(dirpath: &Path) -> Result<Option<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dirpath)? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path);
        }
    }

    for index_name in INDEX_NAMES {
        let found = files.iter().find(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(index_name))
        });
        if let Some(path) = found {
            return Ok(Some(path.clone()));
        }
    }

    Ok(None)
}

fn to_summary_path(src_dir: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(src_dir)?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn collect_files_and_dirs(root_dir: &Path, src_dir: &Path) -> Result<Vec<Entry>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root_dir)? {
        paths.push(entry?.path());
    }
    paths.sort_by(compare_paths);

    let mut entries = Vec::new();
    for item_path in paths {
        let Some(item_name) = item_path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if item_path.is_dir() {
            if !should_include_dir(item_name) {
                continue;
            }

            if let Some(index_file) = find_index_file(&item_path)? {
                entries.push(Entry::Dir {
                    title: extract_title_from_md(&index_file),
                    path: to_summary_path(src_dir, &index_file)?,
                    children: collect_files_and_dirs(&item_path, src_dir)?,
                });
            }
            continue;
        }

        if !item_path.is_file() {
            continue;
        }

        if item_path
            .extension()
            .and_then(|value| value.to_str())
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("md"))
        {
            continue;
        }

        if INDEX_NAMES
            .iter()
            .any(|index_name| item_name.eq_ignore_ascii_case(index_name))
        {
            continue;
        }

        if item_name.eq_ignore_ascii_case("SUMMARY.md") || !should_include_file(item_name) {
            continue;
        }

        entries.push(Entry::File {
            title: extract_title_from_md(&item_path),
            path: to_summary_path(src_dir, &item_path)?,
        });
    }

    Ok(entries)
}

fn compare_paths(left: &PathBuf, right: &PathBuf) -> Ordering {
    match (left.is_dir(), right.is_dir()) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }

    let left_name = left
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let right_name = right
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();

    natural_cmp(&left_name, &right_name)
}

fn natural_cmp(left: &str, right: &str) -> Ordering {
    let left_parts = natural_parts(left);
    let right_parts = natural_parts(right);

    for (left, right) in left_parts.iter().zip(right_parts.iter()) {
        let ordering = match (left, right) {
            (NaturalPart::Number(left), NaturalPart::Number(right)) => left.cmp(right),
            (NaturalPart::Text(left), NaturalPart::Text(right)) => left.cmp(right),
            (NaturalPart::Number(_), NaturalPart::Text(_)) => Ordering::Less,
            (NaturalPart::Text(_), NaturalPart::Number(_)) => Ordering::Greater,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left_parts.len().cmp(&right_parts.len())
}

#[derive(Debug, Eq, PartialEq)]
enum NaturalPart {
    Text(String),
    Number(u64),
}

fn natural_parts(value: &str) -> Vec<NaturalPart> {
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut in_number: Option<bool> = None;

    for ch in value.chars() {
        let is_number = ch.is_ascii_digit();
        match in_number {
            Some(current) if current == is_number => buffer.push(ch),
            Some(current) => {
                parts.push(to_natural_part(&buffer, current));
                buffer.clear();
                buffer.push(ch);
                in_number = Some(is_number);
            }
            None => {
                buffer.push(ch);
                in_number = Some(is_number);
            }
        }
    }

    if let Some(current) = in_number {
        parts.push(to_natural_part(&buffer, current));
    }

    parts
}

fn to_natural_part(value: &str, is_number: bool) -> NaturalPart {
    if is_number {
        NaturalPart::Number(value.parse().unwrap_or(0))
    } else {
        NaturalPart::Text(value.to_string())
    }
}

fn generate_summary(
    src_dir: &Path,
    entries: &[Entry],
    root_index: Option<&Path>,
) -> Result<String> {
    let mut lines = vec!["# Summary".to_string(), String::new()];

    if let Some(root_index) = root_index {
        let root_title = extract_title_from_md(root_index);
        let root_rel_path = to_summary_path(src_dir, root_index)?;
        lines.push(format!("* [{root_title}]({root_rel_path})"));
        lines.push(String::new());
    }

    lines.extend(generate_summary_content(entries, 0));
    Ok(format!("{}\n", lines.join("\n")))
}

fn generate_summary_content(entries: &[Entry], depth: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let indent = "  ".repeat(depth);

    for entry in entries {
        match entry {
            Entry::Dir {
                title,
                path,
                children,
            } => {
                lines.push(format!("{indent}* [{title}]({path})"));
                lines.extend(generate_summary_content(children, depth + 1));
            }
            Entry::File { title, path } => {
                lines.push(format!("{indent}* [{title}]({path})"));
            }
        }
    }

    lines
}

fn count_entries(entries: &[Entry]) -> usize {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Dir { children, .. } => 1 + count_entries(children),
            Entry::File { .. } => 1,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cleans_markdown_title() {
        assert_eq!(
            clean_markdown_title(" `foo` and [bar](https://example.com) ### "),
            "foo and bar"
        );
    }

    #[test]
    fn compares_natural_names() {
        assert_eq!(natural_cmp("chapter2.md", "chapter10.md"), Ordering::Less);
    }

    #[test]
    fn generates_summary_from_tree() -> Result<()> {
        let temp = temp_dir();
        let src = temp.join("src");
        fs::create_dir_all(src.join("part"))?;
        fs::write(src.join("README.md"), "# Book\n")?;
        fs::write(src.join("part").join("index.md"), "# Part\n")?;
        fs::write(src.join("part").join("chapter-2.md"), "# Chapter Two\n")?;

        let entries = collect_files_and_dirs(&src, &src)?;
        let root_index = find_index_file(&src)?;
        let summary = generate_summary(&src, &entries, root_index.as_deref())?;

        assert!(summary.contains("* [Book](README.md)"));
        assert!(summary.contains("* [Part](part/index.md)"));
        assert!(summary.contains("  * [Chapter Two](part/chapter-2.md)"));
        assert_eq!(
            count_entries(&entries) + usize::from(root_index.is_some()),
            3
        );

        fs::remove_dir_all(temp)?;
        Ok(())
    }

    #[test]
    fn summary_md_as_index() -> Result<()> {
        let temp = temp_dir();
        let src = temp.join("src");
        fs::create_dir_all(src.join("appendix"))?;
        fs::write(src.join("README.md"), "# Book\n")?;
        fs::write(src.join("appendix").join("SUMMARY.md"), "# Appendix\n")?;
        fs::write(src.join("appendix").join("notes.md"), "# Notes\n")?;

        let entries = collect_files_and_dirs(&src, &src)?;
        let root_index = find_index_file(&src)?;
        let summary = generate_summary(&src, &entries, root_index.as_deref())?;

        assert!(summary.contains("* [Appendix](appendix/SUMMARY.md)"));
        assert!(summary.contains("  * [Notes](appendix/notes.md)"));

        fs::remove_dir_all(temp)?;
        Ok(())
    }

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("mdbook-summarizer-test-{nanos}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }
}
