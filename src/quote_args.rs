use std::env;

use crate::shell;

fn shell_quote_self_args_impl(args: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    shell::shell_quote_args(args)
}

fn shell_quote_self_args_as_text_impl(args: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut text = String::new();

    let mut args = args.into_iter();
    if let Some(program) = args.next() {
        text.push_str(program.as_ref());
    }

    const NEWLINE_SEP: &str = " \\\n    ";
    let mut next_sep = " ";

    while let Some(arg) = args.next() {
        if ["-a", "-b", "-c", "-d", "-e", "-A", "-B", "-C", "-D", "-E"].contains(&arg.as_ref()) {
            text.push_str(NEWLINE_SEP);
            text.push_str(&shell::shell_quote(arg.as_ref()));
            if let Some(arg) = args.next() {
                text.push_str(" ");
                text.push_str(&shell::shell_quote(arg.as_ref()));
            }
            next_sep = NEWLINE_SEP;
        } else {
            text.push_str(next_sep);
            next_sep = " ";
            text.push_str(&shell::shell_quote(arg.as_ref()));
        }
    }

    text.push_str("\n");
    text
}

pub(crate) fn shell_quote_self_args() -> String {
    shell_quote_self_args_impl(env::args())
}

pub(crate) fn shell_quote_self_args_as_text() -> String {
    shell_quote_self_args_as_text_impl(env::args())
}

#[cfg(test)]
mod tests {
    use crate::quote_args::shell_quote_self_args_as_text_impl;
    use crate::quote_args::shell_quote_self_args_impl;

    #[test]
    fn test_shell_quote_self_args() {
        assert_eq!(
            "absh -a 'sleep 0.1'",
            shell_quote_self_args_impl(["absh", "-a", "sleep 0.1"])
        );
    }

    #[test]
    fn test_shell_quote_self_args_as_text() {
        assert_eq!(
            "absh --max-time 10 \\\n    -a 'sleep 0.1' \\\n    -b 'sleep 0.2' \\\n    -irm\n",
            shell_quote_self_args_as_text_impl([
                "absh",
                "--max-time",
                "10",
                "-a",
                "sleep 0.1",
                "-b",
                "sleep 0.2",
                "-irm"
            ])
        );
    }
}
