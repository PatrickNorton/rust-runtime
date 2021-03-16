use std::borrow::Cow;

pub fn repr(value: char) -> Cow<'static, str> {
    match value {
        '\\' => "\\\\".into(),
        '"' => "\\\"".into(),
        '\0' => "\\0".into(),
        '\x07' => "\\a".into(),
        '\x08' => "\\b".into(),
        '\x0C' => "\\f".into(),
        '\n' => "\\n".into(),
        '\r' => "\\r".into(),
        '\t' => "\\t".into(),
        '\x0B' => "\\v".into(),
        x if x.is_ascii_graphic() || x == ' ' => x.to_string().into(),
        x if x.is_ascii() => format!("\\x{:02X}", x as u32).into(),
        x => {
            let escaped = value.escape_debug().to_string();
            if !escaped.starts_with("\\u") {
                escaped.into()
            } else {
                match x as u32 {
                    x @ 0..=0xFF => unreachable!(
                        "ASCII characters should already be filtered out (got {:?})",
                        x
                    ),
                    x @ 0x100..=0xFFFF => format!("\\u{:04X}", x).into(),
                    x => format!("\\U{:08X}", x).into(),
                }
            }
        }
    }
}
