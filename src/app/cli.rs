use crate::util::{
    config::{load_language, save_language},
    i18n::Lang,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Launch {
    Run(Lang),
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Run,
    Help,
    Version,
}

pub(crate) fn parse_args<I>(args: I) -> Result<Launch, String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let default_lang = load_language();
    let (action, lang, should_save) = parse_args_with_default(args, default_lang)?;

    if should_save {
        if let Err(err) = save_language(lang) {
            eprintln!("warning: cannot save language config: {err}");
        }
    }

    match action {
        Action::Run => Ok(Launch::Run(lang)),
        Action::Help => {
            print_help(lang);
            Ok(Launch::Exit)
        }
        Action::Version => {
            println!("rfdisk {}", env!("CARGO_PKG_VERSION"));
            Ok(Launch::Exit)
        }
    }
}

fn parse_args_with_default<I>(args: I, default_lang: Lang) -> Result<(Action, Lang, bool), String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut lang = default_lang;
    let mut should_save = false;
    let mut action = Action::Run;
    let mut iter = args.into_iter().map(Into::into).skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => action = Action::Help,
            "-V" | "--version" => action = Action::Version,
            "--lang" => {
                let Some(value) = iter.next() else {
                    return Err("missing value for --lang".to_string());
                };
                lang = parse_lang_arg(&value)?;
                should_save = true;
            }
            value if value.starts_with("--lang=") => {
                let value = value.trim_start_matches("--lang=");
                lang = parse_lang_arg(value)?;
                should_save = true;
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok((action, lang, should_save))
}

pub(crate) fn print_help(lang: Lang) {
    match lang {
        Lang::En => {
            println!(
                "\
rfdisk {version}

A refresh-first cfdisk-like partition table editor for Linux.

Usage:
  rfdisk [OPTIONS]

Options:
  -h, --help              Show this help message
  -V, --version           Show version information
      --lang <LANG>       Select and save UI language: en, zh-CN

Notes:
  Run with sudo on Linux for full disk scanning and write support.
  Set(plan) is visible in the UI but still under development in alpha.
",
                version = env!("CARGO_PKG_VERSION")
            );
        }
        Lang::ZhCn => {
            println!(
                "\
rfdisk {version}

面向 Linux 的 refresh-first、类 cfdisk 分区表编辑工具。

用法:
  rfdisk [OPTIONS]

选项:
  -h, --help              显示帮助信息
  -V, --version           显示版本信息
      --lang <LANG>       选择并保存界面语言: en, zh-CN

说明:
  在 Linux 上建议使用 sudo 运行，以获得完整磁盘扫描和写盘能力。
  Set(plan) 在 alpha 版本中只保留入口，仍在研发中。
",
                version = env!("CARGO_PKG_VERSION")
            );
        }
    }
}

fn parse_lang_arg(value: &str) -> Result<Lang, String> {
    Lang::parse(value)
        .ok_or_else(|| format!("unsupported language: {value}. Supported values: en, zh-CN"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_language() {
        assert_eq!(
            parse_args_with_default(["rfdisk"], Lang::En).unwrap(),
            (Action::Run, Lang::En, false)
        );
    }

    #[test]
    fn parses_lang_option() {
        assert_eq!(
            parse_args_with_default(["rfdisk", "--lang", "zh-CN"], Lang::En).unwrap(),
            (Action::Run, Lang::ZhCn, true)
        );
        assert_eq!(
            parse_args_with_default(["rfdisk", "--lang=en"], Lang::ZhCn).unwrap(),
            (Action::Run, Lang::En, true)
        );
    }

    #[test]
    fn help_uses_selected_language() {
        assert_eq!(
            parse_args_with_default(["rfdisk", "--lang", "zh-CN", "--help"], Lang::En).unwrap(),
            (Action::Help, Lang::ZhCn, true)
        );
    }

    #[test]
    fn rejects_unknown_language() {
        assert!(parse_args_with_default(["rfdisk", "--lang", "jp"], Lang::En).is_err());
    }

    #[test]
    fn help_and_version_exit() {
        assert_eq!(
            parse_args_with_default(["rfdisk", "-h"], Lang::En).unwrap(),
            (Action::Help, Lang::En, false)
        );
        assert_eq!(
            parse_args_with_default(["rfdisk", "-V"], Lang::En).unwrap(),
            (Action::Version, Lang::En, false)
        );
    }
}
