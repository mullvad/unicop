pub fn unicode_notation_to_char(unicode_notation: &str) -> Option<char> {
    let hex_str_number = unicode_notation.strip_prefix("U+")?;
    let int_number = u32::from_str_radix(hex_str_number, 16).ok()?;
    char::from_u32(int_number)
}

pub fn char_to_unicode_notation(c: char) -> String {
    format!("U+{:X}", u32::from(c))
}
