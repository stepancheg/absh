/// Char can be inserted in single quoted strings without escaping.
fn as_is_with_single(c: char) -> bool {
    c != '\'' && !c.is_control()
}

/// Char can be inserted in double quoted strings without escaping.
fn as_is_with_double(c: char) -> bool {
    c != '"' && c != '\\' && c != '$' && c != '`' && !c.is_control()
}

pub fn shell_quote(s: &str) -> String {
    // If the string is empty, we need to quote it.
    if s.is_empty() {
        return "''".to_owned();
    }

    // If a string does not need escaping, just return it.
    if !s.contains(|c| {
        c == '"' || c == '\'' || c == '\"' || c <= ' ' || c == '\\' || c == '$' || c == '`'
    }) {
        return s.to_owned();
    }

    enum State {
        None,
        Single,
        Double,
        Dollar,
    }

    struct Escaper {
        state: State,
        r: String,
    }

    let mut escaper = Escaper {
        state: State::None,
        r: String::new(),
    };

    impl Escaper {
        fn close(&mut self) {
            match &self.state {
                State::None => {}
                State::Single | State::Dollar => {
                    self.r.push('\'');
                }
                State::Double => {
                    self.r.push('"');
                }
            }
            self.state = State::None;
        }

        fn open(&mut self, state: State) {
            self.close();
            match state {
                State::None => {}
                State::Single => {
                    self.r.push('\'');
                }
                State::Double => {
                    self.r.push('"');
                }
                State::Dollar => {
                    self.r.push_str("$'");
                }
            }
            self.state = state;
        }

        fn push_char(&mut self, c: char) {
            match &self.state {
                State::None => {
                    if as_is_with_single(c) {
                        self.open(State::Single);
                        self.push_char(c);
                    } else if as_is_with_double(c) {
                        self.open(State::Double);
                        self.push_char(c);
                    } else {
                        self.open(State::Dollar);
                        self.push_char(c);
                    }
                }
                State::Single => {
                    if as_is_with_single(c) {
                        self.r.push(c);
                    } else {
                        self.close();
                        self.push_char(c);
                    }
                }
                State::Double => {
                    if as_is_with_double(c) {
                        self.r.push(c);
                    } else {
                        self.close();
                        self.push_char(c);
                    }
                }
                State::Dollar => {
                    if as_is_with_single(c) {
                        self.open(State::Single);
                        self.push_char(c);
                    } else if as_is_with_double(c) {
                        self.open(State::Double);
                        self.push_char(c);
                    } else {
                        match c {
                            '\\' => self.r.push_str("\\\\"),
                            '\n' => self.r.push_str("\\n"),
                            '\r' => self.r.push_str("\\r"),
                            '\t' => self.r.push_str("\\t"),
                            c => self.r.push_str(&format!("\\{:03o}", c as u8)),
                        }
                    }
                }
            }
        }

        fn finish(mut self) -> String {
            self.close();
            self.r
        }
    }

    let prefix_single = s.chars().take_while(|c| as_is_with_single(*c)).count();
    let prefix_double = s.chars().take_while(|c| as_is_with_double(*c)).count();

    if prefix_single != 0 && prefix_single >= prefix_double {
        escaper.open(State::Single);
    } else if prefix_double != 0 {
        escaper.open(State::Double);
    }

    for c in s.chars() {
        escaper.push_char(c);
    }

    escaper.finish()
}

pub fn shell_quote_args(args: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    args.into_iter()
        .map(|s| shell_quote(s.as_ref()))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::shell::shell_quote;
    use crate::shell::shell_quote_args;

    #[test]
    fn test_shell_quote() {
        assert_eq!("''", shell_quote(""));
        assert_eq!("xy", shell_quote("xy"));
        assert_eq!("'x y'", shell_quote("x y"));
        assert_eq!("'x$y'", shell_quote("x$y"));
        assert_eq!("'x\\y'", shell_quote("x\\y"));
        assert_eq!("'x\"y'", shell_quote("x\"y"));
        assert_eq!("\"x'y\"", shell_quote("x'y"));
        assert_eq!("\"x'y\"'\"z'", shell_quote("x'y\"z"));
        assert_eq!("'x'$'\\n''y'", shell_quote("x\ny"));
        assert_eq!("'x'$'\\r''y'", shell_quote("x\ry"));
        assert_eq!("'x'$'\\001''y'", shell_quote("x\x01y"));
    }

    #[test]
    fn test_shell_quote_args() {
        assert_eq!("echo '$A'", shell_quote_args(&["echo", "$A"]));
    }
}
