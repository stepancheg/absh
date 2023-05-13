use std::env;

use crate::shell::shell_quote_args;

fn shell_quote_self_args_impl(args: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    shell_quote_args(args)
}

pub(crate) fn shell_quote_self_args() -> String {
    shell_quote_self_args_impl(env::args())
}

#[cfg(test)]
mod tests {
    use crate::quote_args::shell_quote_self_args_impl;

    #[test]
    fn test_shell_quote_self_args() {
        assert_eq!(
            "foo --bar 'b z'",
            shell_quote_self_args_impl(["foo", "--bar", "b z"])
        );
    }
}
