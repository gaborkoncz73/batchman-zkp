//! Token text → value: quoted atoms and strings, escapes, integer literals.

use anyhow::{Context, Result, bail};
use num_bigint::BigInt;

/// Converts a (possibly radix-prefixed) integer literal to canonical decimal.
pub(super) fn canonical_integer(text: &str, radix: u32, prefix: &str) -> Result<String> {
    let digits = text
        .strip_prefix(prefix)
        .context("integer has an invalid radix prefix")?;
    let value = BigInt::parse_bytes(digits.as_bytes(), radix)
        .context("integer literal is outside the supported syntax")?;
    Ok(value.to_string())
}

/// Decodes a `0'c` character-code constant to its code point.
pub(super) fn decode_character_code(text: &str) -> Result<u32> {
    let encoded = text
        .strip_prefix("0'")
        .context("invalid character-code constant")?;
    let decoded = decode_content(encoded, None)?;
    let mut characters = decoded.chars();
    let character = characters.next().context("empty character-code constant")?;
    if characters.next().is_some() {
        bail!("character-code constant must decode to exactly one character");
    }
    Ok(character as u32)
}

/// Strips the surrounding `quote` characters and decodes the content.
pub(super) fn decode_quoted(text: &str, quote: char) -> Result<String> {
    if !text.starts_with(quote) || !text.ends_with(quote) || text.len() < 2 {
        bail!("invalid quoted literal");
    }
    let inner = &text[quote.len_utf8()..text.len() - quote.len_utf8()];
    decode_content(inner, Some(quote))
}

/// Decodes ISO Prolog escapes and doubled quotes inside a quoted token.
fn decode_content(input: &str, doubled_quote: Option<char>) -> Result<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::new();
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        if current != '\\' {
            output.push(current);
            let doubled =
                Some(current) == doubled_quote && chars.get(index + 1).copied() == doubled_quote;
            index += if doubled { 2 } else { 1 };
            continue;
        }

        let (decoded, next) = match *chars.get(index + 1).context("trailing escape")? {
            // Line continuation: the escaped newline produces no character.
            '\n' => (None, index + 2),
            '\r' if chars.get(index + 2) == Some(&'\n') => (None, index + 3),
            'x' => decode_radix_escape(&chars, index + 2, 16)?,
            '0'..='7' => decode_radix_escape(&chars, index + 1, 8)?,
            escaped => {
                let character = match escaped {
                    'a' => '\u{0007}',
                    'b' => '\u{0008}',
                    'f' => '\u{000c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\u{000b}',
                    other => other,
                };
                (Some(character), index + 2)
            }
        };
        output.extend(decoded);
        index = next;
    }
    Ok(output)
}

/// Decodes `\x41\` or `\101\` starting at the first digit; returns the next index.
fn decode_radix_escape(chars: &[char], start: usize, radix: u32) -> Result<(Option<char>, usize)> {
    let end = start
        + chars[start..]
            .iter()
            .position(|&character| character == '\\')
            .context("numeric escape must end with a backslash")?;
    if end == start {
        bail!("numeric escape must end with a backslash");
    }
    let digits: String = chars[start..end].iter().collect();
    let value = u32::from_str_radix(&digits, radix).context("invalid numeric escape")?;
    let character = char::from_u32(value).context("numeric escape is not a Unicode scalar")?;
    Ok((Some(character), end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_literals_are_decoded() {
        assert_eq!(decode_quoted("'can''t'", '\'').unwrap(), "can't");
        assert_eq!(decode_quoted(r#""line\nnext""#, '"').unwrap(), "line\nnext");
        assert_eq!(decode_quoted(r#"'\x41\'"#, '\'').unwrap(), "A");
    }

    #[test]
    fn radix_integers_are_canonical_decimal() {
        assert_eq!(canonical_integer("0b00101", 2, "0b").unwrap(), "5");
        assert_eq!(canonical_integer("0x10", 16, "0x").unwrap(), "16");
    }
}
