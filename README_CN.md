# mdbook-summarizer

[English](README.md)

从源目录树自动生成 mdBook 的 `SUMMARY.md`。

这是一个用 Rust 编写的 CLI 工具，移植自原有的 `generate_summary.py` 工作流。它扫描 Markdown 文件目录，生成结构正确的 `SUMMARY.md`，支持自然数排序、自动提取标题和嵌套层级。

## 安装

```sh
cargo install mdbook-summarizer
```

或从源码构建：

```sh
git clone https://github.com/wustites/mdbook-summarizer
cd mdbook-summarizer
cargo build --release
```

## 用法

```sh
# 扫描 src/ 目录，写入 src/SUMMARY.md（默认行为）
mdbook-summarizer

# 指定自定义源目录
mdbook-summarizer --src docs

# 指定自定义输出路径
mdbook-summarizer --src src --output SUMMARY.md

# 预览生成内容，不写入文件
mdbook-summarizer --dry-run

# 为缺少索引文件的目录自动生成 README.md
mdbook-summarizer --auto-readme

# 验证输出文件是否为最新（适用于 CI）
mdbook-summarizer --check
```

## 参数

| 参数 | 说明 |
|------|------|
| `--src <DIR>` | 要扫描的源目录（默认：`src`） |
| `-o, --output <FILE>` | 输出文件路径（默认：`<src>/SUMMARY.md`） |
| `--auto-readme` | 为缺少索引文件的目录自动生成 `README.md`（内容为 `# 目录名`） |
| `--dry-run` | 将生成内容打印到标准输出，不写入文件 |
| `--check` | 输出文件过时则报错退出 |
| `-h, --help` | 打印帮助信息 |
| `-V, --version` | 打印版本号 |

## 行为说明

- 将 `README.md`、`index.md` 或 `SUMMARY.md` 视为目录的索引入口。
- `--auto-readme` 为没有索引文件的目录自动生成 `README.md`，内容为 `# 目录名`。
- 提取第一个一级 Markdown 标题（`# 标题`）作为条目标题。
- 未找到标题时，将文件名转换为首字母大写作为回退标题。
- 自动去除标题中的行内代码和 Markdown 链接。
- 跳过 `SUMMARY.md`、点目录（`.git`、`.github` 等）以及常见临时/备份文件
  （`*.bak`、`*.tmp`、`*~`，文件名含 `backup`、`old`、`draft` 的文件）。
- 目录排在文件之前，按自然数排序（`chapter2` 排在 `chapter10` 前面）。

## CI 集成

在 CI 流水线中使用 `--check` 确保 `SUMMARY.md` 保持同步：

```sh
mdbook-summarizer --check || (echo "SUMMARY.md 已过时" && exit 1)
```

## 许可证

MIT
