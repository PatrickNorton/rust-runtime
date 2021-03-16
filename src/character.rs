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
        x if x.is_ascii_graphic() || x == ' ' => x.to_string().into(),
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
