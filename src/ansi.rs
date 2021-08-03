use regex::Regex;

/// Green color
pub const GREEN: &str = "\x1B[32m";
/// Red color
pub const RED: &str = "\x1B[31m";
/// Yellow color
pub const YELLOW: &str = "\x1B[33m";
/// Blue color
pub const BLUE: &str = "\x1B[34m";
/// Magenta color
pub const MAGENTA: &str = "\x1B[35m";
/// Reset color
pub const RESET: &str = "\x1B[0m";

// https://en.wikipedia.org/wiki/ANSI_escape_code#CSIsection
pub fn strip_csi(s: &str) -> String {
    Regex::new("\x1b\\[[0-9]+[a-zA-Z]")
        .unwrap()
        .replace_all(s, "")
        .into_owned()
}

#[cfg(test)]
mod test {
    use crate::ansi::strip_csi;

    #[test]
    fn test() {
        assert_eq!("A\nB\n", strip_csi("\x1B[32mA\x1B[0m\n\x1B[31mB\x1B[0m\n"));
    }
}
