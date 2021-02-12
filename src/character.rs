use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::collections::HashSet;
use unic_ucd_category::GeneralCategory;

pub fn repr(value: char) -> Cow<'static, str> {
    static PRINTABLE_CLASSES: Lazy<HashSet<GeneralCategory>> = Lazy::new(|| {
        hash_set!(
            GeneralCategory::UppercaseLetter,
            GeneralCategory::LowercaseLetter,
            GeneralCategory::TitlecaseLetter,
            GeneralCategory::ModifierLetter,
            GeneralCategory::OtherLetter,
            GeneralCategory::NonspacingMark,
            GeneralCategory::EnclosingMark,
            GeneralCategory::DecimalNumber,
            GeneralCategory::LetterNumber,
            GeneralCategory::OtherNumber,
            GeneralCategory::SpaceSeparator,
            GeneralCategory::DashPunctuation,
            GeneralCategory::OpenPunctuation,
            GeneralCategory::ClosePunctuation,
            GeneralCategory::ConnectorPunctuation,
            GeneralCategory::OtherPunctuation,
            GeneralCategory::MathSymbol,
            GeneralCategory::CurrencySymbol,
            GeneralCategory::ModifierSymbol,
            GeneralCategory::OtherSymbol,
            GeneralCategory::InitialPunctuation,
            GeneralCategory::FinalPunctuation,
        )
    });

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
        x if (&*PRINTABLE_CLASSES).contains(&GeneralCategory::of(x)) => String::from(x).into(),
        x => match x as u32 {
            x @ 0..=0xFF => format!("\\x{:02X}", x).into(),
            x @ 0x100..=0xFFFF => format!("\\u{:04X}", x).into(),
            x => format!("\\U{:08X}", x).into(),
        },
    }
}
