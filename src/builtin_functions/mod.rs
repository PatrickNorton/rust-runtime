pub mod bool_fn;
pub mod char_fn;
pub mod dec_fn;
pub mod int_fn;
pub mod null_fn;
pub mod option_fn;
pub mod string_fn;
pub mod tuple_fn;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Encoding {
    Ascii,
    Utf8,
    Utf16Be,
    Utf16Le,
    Utf32Be,
    Utf32Le,
}

impl Encoding {
    pub fn from_str(value: &str) -> Result<Encoding, &str> {
        match &*value.to_lowercase() {
            "ascii" => Result::Ok(Encoding::Ascii),
            "utf-8" => Result::Ok(Encoding::Utf8),
            "utf-16" | "utf-16le" => Result::Ok(Encoding::Utf16Le),
            "utf-16be" => Result::Ok(Encoding::Utf16Be),
            "utf-32" | "utf-32le" => Result::Ok(Encoding::Utf32Le),
            "utf-32be" => Result::Ok(Encoding::Utf32Be),
            _ => Result::Err(value),
        }
    }
}
