mod clipboard;

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

#[derive(Default)]
struct Options {
    quiet: bool,
    full: bool,
}

const HELP: &str = "zcco — z-copy command output

用法: command | zcco [选项]

将标准输入的 UTF-8 文本复制到 macOS 剪贴板，默认 trim 首尾空白。
默认按原样打印输入，随后另起一行显示 zcco copied.

选项:
  -q, --quiet    静默模式，不打印内容和成功提示（错误仍输出到 stderr）
  -f, --full     复制完整输入，不去掉首尾空白
  -h, --help     显示帮助
  -V, --version  显示版本

示例:
  echo 'Hello' | zcco
  git diff | zcco --quiet

仅支持 macOS，直接调用系统剪贴板接口，无需安装外部命令。
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "zcco: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut options = Options::default();
    for arg in std::env::args_os().skip(1) {
        match arg.to_str() {
            Some("-q" | "--quiet") => options.quiet = true,
            Some("-f" | "--full") => options.full = true,
            Some("-h" | "--help") => return output(HELP.as_bytes()),
            Some("-V" | "--version") => {
                return output(concat!("zcco ", env!("CARGO_PKG_VERSION"), "\n").as_bytes());
            }
            _ => {
                return Err(format!(
                    "未知参数 {}，使用 --help 查看帮助",
                    arg.to_string_lossy()
                ));
            }
        }
    }

    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err("请通过管道或重定向提供输入，例如 echo 'Hello' | zcco".into());
    }
    copy_input(stdin.lock(), io::stdout().lock(), options, clipboard::copy)
}

fn copy_input(
    mut source: impl Read,
    mut stdout: impl Write,
    options: Options,
    copy: impl FnOnce(&str) -> Result<(), String>,
) -> Result<(), String> {
    let mut input = String::new();
    source
        .read_to_string(&mut input)
        .map_err(|error| format!("读取 UTF-8 标准输入失败: {error}"))?;
    copy(if options.full { &input } else { input.trim() })?;
    if !options.quiet {
        let mut print = || -> io::Result<()> {
            stdout.write_all(input.as_bytes())?;
            if !input.is_empty() && !input.ends_with('\n') {
                writeln!(stdout)?;
            }
            writeln!(stdout, "zcco copied.")
        };
        print().map_err(|error| format!("内容已复制，但打印失败: {error}"))?;
    }
    Ok(())
}

fn output(bytes: &[u8]) -> Result<(), String> {
    io::stdout()
        .lock()
        .write_all(bytes)
        .map_err(|error| format!("打印失败: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_original_input_but_copies_trimmed_text() {
        let input = "\r\n \t你好\nworld \t\r\n";
        let mut copied = String::new();
        let mut stdout = Vec::new();
        copy_input(input.as_bytes(), &mut stdout, Options::default(), |text| {
            copied = text.to_owned();
            Ok(())
        })
        .unwrap();
        assert_eq!(copied, "你好\nworld");
        assert_eq!(stdout, "\r\n \t你好\nworld \t\r\nzcco copied.\n".as_bytes());
    }

    #[test]
    fn full_mode_copies_all_original_whitespace() {
        let input = "\n \t你好 \r\n";
        let mut copied = String::new();
        let mut stdout = Vec::new();
        copy_input(
            input.as_bytes(),
            &mut stdout,
            Options {
                full: true,
                quiet: false,
            },
            |text| {
                copied = text.to_owned();
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(copied, input);
        assert_eq!(stdout, "\n \t你好 \r\nzcco copied.\n".as_bytes());
    }

    #[test]
    fn adds_a_separator_only_when_original_has_no_final_newline() {
        for (input, expected) in [
            ("hello", "hello\nzcco copied.\n"),
            ("hello\n\n", "hello\n\nzcco copied.\n"),
            ("", "zcco copied.\n"),
        ] {
            let mut stdout = Vec::new();
            copy_input(
                input.as_bytes(),
                &mut stdout,
                Options::default(),
                |_| Ok(()),
            )
            .unwrap();
            assert_eq!(stdout, expected.as_bytes());
        }
    }

    #[test]
    fn quiet_mode_still_applies_full_or_trim_selection() {
        for (full, expected) in [(false, "hello"), (true, " hello\n")] {
            let mut copied = String::new();
            let mut stdout = Vec::new();
            copy_input(
                b" hello\n".as_slice(),
                &mut stdout,
                Options { quiet: true, full },
                |text| {
                    copied = text.to_owned();
                    Ok(())
                },
            )
            .unwrap();
            assert_eq!(copied, expected);
            assert!(stdout.is_empty());
        }
    }

    #[test]
    fn whitespace_only_input_copies_empty_text_by_default() {
        let mut copied = None;
        let mut stdout = Vec::new();
        copy_input(
            " \t\u{3000}\r\n".as_bytes(),
            &mut stdout,
            Options::default(),
            |text| {
                copied = Some(text.to_owned());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(copied.as_deref(), Some(""));
        assert_eq!(stdout, " \t\u{3000}\r\nzcco copied.\n".as_bytes());
    }

    #[test]
    fn copy_failure_never_prints_success_even_when_quiet() {
        for quiet in [false, true] {
            let mut stdout = Vec::new();
            let result = copy_input(
                b"hello".as_slice(),
                &mut stdout,
                Options { quiet, full: false },
                |_| Err("剪贴板不可用".into()),
            );
            assert_eq!(result.unwrap_err(), "剪贴板不可用");
            assert!(stdout.is_empty());
        }
    }

    #[test]
    fn invalid_utf8_does_not_reach_clipboard() {
        let mut stdout = Vec::new();
        let result = copy_input([0xff].as_slice(), &mut stdout, Options::default(), |_| {
            panic!("invalid UTF-8 must not be copied");
        });
        assert!(result.is_err());
        assert!(stdout.is_empty());
    }
}
