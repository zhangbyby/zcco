# zcco — z-copy command output

使用 Rust 编写的 macOS 命令行工具，通过管道接收 UTF-8 文本并复制到系统剪贴板。直接调用 `NSPasteboard`，无需 `pbcopy` 或其他外部命令，支持 Apple Silicon 和 Intel Mac。

## 安装

首个稳定版发布到 Homebrew tap 后，可以安装预编译版本：

```sh
brew install zhangbyby/tap/zcco
```

从源码安装：

需要 macOS、Xcode Command Line Tools 和 Rust 1.85 或更新版本：

```sh
cargo install --path .
```

安装后确保 `~/.cargo/bin` 位于 `PATH` 中。也可在项目内直接运行：

```sh
echo 'Hello' | cargo run --quiet --
```

## 使用

```sh
echo 'Hello' | zcco
git diff | zcco --quiet
cat README.md | zcco --full
zcco < README.md
```

默认打印输入原文（保留前后换行、空格、制表符等），最后另起一行显示：

```text
Hello
zcco copied.
```

打印与复制互相独立：默认只将 `input.trim()` 后的文本复制到剪贴板，去掉首尾 Unicode 空白字符，包括空格、制表符、换行和全角空格；中间内容保持原样。使用 `-f` / `--full` 时复制完整原文。

| 参数 | 作用 |
| --- | --- |
| `-f`, `--full` | 复制完整输入，保留首尾全部空白 |
| `-q`, `--quiet` | 不打印原文和成功提示；错误仍输出到 stderr |
| `-h`, `--help` | 显示帮助 |
| `-V`, `--version` | 显示版本 |

```sh
printf '\n  Hello  \n\n' | zcco
# stdout 保留原文中的首尾空白；剪贴板内容为 Hello

printf '\n  Hello  \n\n' | zcco --full
# stdout 保留原文中的首尾空白；剪贴板内容也保留完整原文

printf '\n  Hello  \n\n' | zcco -f -q
# 复制完整原文，成功时不输出
```

只有复制成功后才打印。原文末尾没有 `\n` 且内容非空时，为让 `zcco copied.` 独占一行会追加一个分隔换行；已有末尾换行则直接追加提示。空输入只显示 `zcco copied.`。提示文字和额外的分隔换行不会写入剪贴板。

默认情况下，空输入或纯空白输入会复制空文本；`--full` 则原样复制空白。输入必须是 UTF-8，全部读取到内存后再复制。没有管道或重定向时，会提示用法并退出。

成功返回退出码 `0`；参数错误、输入读取失败、剪贴板操作失败或输出写入失败返回非零退出码。需要能访问 macOS 系统剪贴板服务的桌面会话。

## 构建

构建当前 Mac 的发布版本：

```sh
cargo build --release --locked
```

可执行文件位于 `target/release/zcco`。

在安装了 Xcode Command Line Tools 的 macOS 上，交叉构建两种 Mac 架构：

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --locked --target aarch64-apple-darwin
cargo build --release --locked --target x86_64-apple-darwin
```

产物位于 `target/<目标>/release/zcco`。CI 在 macOS runner 上运行检查和测试，并上传这两种目标的构建产物。当前不支持 Linux。

## 开发检查

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

原生接口使用 `objc2`、`objc2-app-kit` 和 `objc2-foundation` 绑定。测试覆盖原文输出、trim/full、静默、参数解析及错误处理；原生测试通过独立命名剪贴板验证中文、空文本及大段内容，不修改用户的通用系统剪贴板。原生测试需要访问桌面剪贴板服务。

Formula 生成脚本使用 Python 3.11+ 标准库，可运行以下检查：

```sh
python3 -m unittest discover -s tests -p 'test_*.py'
goreleaser check
actionlint
```

## 发布与 Homebrew tap

参考 `zuuid` 的标签发布流程，推送与 `Cargo.toml` 版本一致的 `v*` 标签后，GitHub Actions 将：

1. 在 macOS 上运行格式检查、Clippy 和测试。
2. 使用 GoReleaser 2.20.2 构建 Apple Silicon、Intel 两种 macOS 版本。
3. 发布 GitHub Release，上传两个 `.tar.gz` 文件及 `checksums.txt`。
4. 根据实际发布包计算 SHA-256，生成包含两种架构的 Formula，并提交到 `zhangbyby/homebrew-tap` 的 `Formula/zcco.rb`。

例如发布当前版本：

```sh
git tag v0.1.0
git push origin v0.1.0
```

后续发布需先更新 `Cargo.toml` 和 `Cargo.lock` 中的版本并提交，再创建对应标签。预发布标签（如 `v0.2.0-rc.1`）仅发布 GitHub Release，不更新稳定版 tap。

发布流程使用内置 `GITHUB_TOKEN` 写入当前仓库的 Release。tap 写入使用 `HOMEBREW_TAP_SSH_KEY` Actions Secret：对应的公钥需作为 **允许写入** 的 deploy key 配置在 `zhangbyby/homebrew-tap`。这把专用密钥无需账号级 PAT，也不依赖 `zuuid` 的 Secret。

流水线沿用 Formula 安装方式，未使用 GoReleaser 已弃用的 `brews` 配置。Formula 中的 `brew test` 只检查版本，不修改用户剪贴板。
