/// Turns a hex string into a vector of bytes.
/// 
/// ```
/// # use myhex::hex;
/// assert_eq!(&hex("010aff"), &[1, 10, 255]);
/// ```
/// 
/// Panics if the input string's length is not a multiple of 2 or if it 
/// contains characters other than `0-9`, `a-f` and `A-F`.
/// ```
pub fn hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    let output_len = bytes.len() / 2;

    (0..output_len).map(|idx| {
        let msb = digit(bytes[idx * 2]);
        let lsb = digit(bytes[idx * 2 + 1]);
        (msb<<4) + lsb
    }).collect()
}

/// Turns a single ascii character into the number it represents.
/// 
/// Panics for characters other than `0-9`, `a-f` and `A-F`.
fn digit(ascii_char: u8) -> u8 {
    match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'f' => ascii_char - b'a' + 10,
        b'A'..=b'F' => ascii_char - b'A' + 10,
        _ => panic!("Invalid character"),
    }
}

#[test]
fn test_basic() {
    assert_eq!(&hex("010aff"), &[1, 10, 255]);
    // lower-case letters
    assert_eq!(&hex("abcd"), &[0xab, 0xcd]);
    // upper-case letters
    assert_eq!(&hex("ABCD"), &[0xab, 0xcd]);
    // mixed-case letters
    assert_eq!(&hex("AbcD"), &[0xab, 0xcd]);
}

#[test]
#[should_panic(expected = "Length needs to be even")]
fn test_invalic_length() {
    hex("123");
}

#[test]
#[should_panic(expected = "Invalid character")]
fn test_invalid_character() {
    hex("12JQÖ");
}

#[test]
fn test_digit() {
    assert_eq!(digit(b'0'), 0);
    assert_eq!(digit(b'a'), 10);
    assert_eq!(digit(b'F'), 15);
}
