use std::borrow::Cow;

pub fn repr(value: char) -> Cow<'static, str> {
    match value {
        '\\' => r"\\".into(),
        '"' => r#"\""#.into(),
        '\0' => r"\0".into(),
        '\x07' => r"\a".into(),
        '\x08' => r"\b".into(),
        '\x0C' => r"\f".into(),
        '\n' => r"\n".into(),
        '\r' => r"\r".into(),
        '\t' => r"\t".into(),
        '\x0B' => r"\v".into(),
        x @ ' '..='~' => x.to_string().into(), // graphic ASCII characters
        x if x.is_ascii() => format!(r"\x{:02X}", x as u32).into(),
        x => {
            let escaped = value.escape_debug().to_string();
            if !escaped.starts_with(r"\u") {
                escaped.into()
            } else {
                match x as u32 {
                    x @ 0..=0xFF => unreachable!(
                        "ASCII characters should already be filtered out (got {:?})",
                        x
                    ),
                    x @ 0x100..=0xFFFF => format!(r"\u{:04X}", x).into(),
                    x => format!(r"\U{:08X}", x).into(),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::character::repr;

    #[test]
    fn backslash() {
        assert_eq!(&repr('\\'), r"\\");
    }

    #[test]
    fn quote() {
        assert_eq!(&repr('"'), "\\\"");
    }

    #[test]
    fn printable_ascii() {
        assert_eq!(&repr('b'), "b");
    }

    #[test]
    fn ascii() {
        assert_eq!(&repr('\x03'), r"\x03");
    }

    #[test]
    fn printable() {
        assert_eq!(&repr('á'), "á");
    }

    #[test]
    fn bmp() {
        // First character in the Private Use area
        assert_eq!(&repr('\u{E000}'), r"\uE000");
    }

    #[test]
    fn non_bmp() {
        assert_eq!(&repr('\u{E0030}'), r"\U000E0030");
    }
}
