/// Turns a hex string into a vector of bytes.
/// 
/// See also the `hex!()` macro, which wraps this function and automatically
/// fills the correct value of the generic parameter `N`.
/// 
/// ```
/// # use myhex::hex;
/// let bytes = hex("010aff");
/// assert_eq!(bytes, [1, 10, 255]);
///
/// // with type annotations
/// let bytes: [u8; 3] = hex::<3>("010AFF");
/// assert_eq!(bytes, [1, 10, 255]);
/// 
/// // usage as a constant
/// const BYTES: [u8; 3] = hex("010AFf");
/// assert_eq!(BYTES, [1, 10, 255]);
/// ```
/// 
/// Panics if the input string's length is not a multiple of 2, if the
/// generic parameter `N` is not half of the input length or if it 
/// contains characters other than `0-9`, `a-f` and `A-F`.
/// 
/// ```should_panic
/// # use myhex::hex;
/// // invalid input length
/// hex::<1>("111");
/// ```
/// 
/// ```should_panic
/// # use myhex::hex;
/// // generic parameter `N` is not half of the input size.
/// hex::<3>("1111");
/// ```
/// 
/// ```should_panic
/// # use myhex::hex;
/// // input contains invalid character `"X"`
/// hex::<2>("11X1");
/// ```
/// 
/// When using `hex()` in a constant context, panics will become 
/// compilation errors:
/// 
/// ```compile_fail
/// # use myhex::hex;
/// // input contains invalid character `"X"`
/// const X: [u8; 2] = hex("11X1");
/// ```
pub const fn hex<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    assert!(bytes.len() == N * 2, "Invalid length (`N * 2 == s.len()` not satisfied).");
    
    let mut arr = [0; N];
    let mut idx = 0;
    while idx < N {
        let msb = ascii_char_to_num(bytes[idx * 2]);
        let lsb = ascii_char_to_num(bytes[idx * 2 + 1]);
        arr[idx] = (msb<<4) + lsb;
        idx += 1;
    }
    arr
}

/// Turns a single ascii character into the number it represents.
///
/// Panics for characters other than `0-9`, `a-f` and `A-F`.
const fn ascii_char_to_num(ascii_char: u8) -> u8 {
    match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'f' => ascii_char - b'a' + 10,
        b'A'..=b'F' => ascii_char - b'A' + 10,
        _ => panic!("Invalid character"),
    }
}

/// Turns a hex string into an of bytes at compile-time.
/// 
/// Compared to to using the `hex()` function directly, this macro ensures 
/// that the transformation is evaluated at compile time, even when the result
/// is used in a regular (non-const) variable. This macro also makes sure that
/// the generic paramter `N` of `hex()` is set correctly.
/// 
/// ```rust
/// # use myhex::hex;
/// assert_eq!(&hex!("010aff"), &[1, 10, 255]);
/// 
/// // declaring a constant
/// hex!{
///     const MY_BYTES = "123456";
/// }
/// assert_eq!(MY_BYTES, [0x12, 0x34, 0x56]);
/// 
/// // declaring a variable (evaluation still happens at compile-time)
/// let my_bytes = hex!("123456");
/// assert_eq!(my_bytes, [0x12, 0x34, 0x56]);
/// ```
/// 
/// Using invalid characters or a string which doesn't have even length 
/// results in a compilation error:
/// 
/// ```compile_fail
/// // invalid length
/// hex!("123");
/// ```
/// 
/// ```compile_fail
/// // invalid character `Q`
/// hex!("123Q");
/// ```
#[macro_export]
macro_rules! hex {
    (const $name:ident = $hex:expr;) => {
        const $name: [u8; $hex.len() / 2] = $crate::hex($hex);
    };
    ($hex:expr) => {
        {
            const STR: &'static str = $hex;
            const X: [u8; {STR.len() / 2}] = $crate::hex(STR);
            X
        }
    };
}


#[test]
fn test_ascii_char_to_num() {
    assert_eq!(ascii_char_to_num(b'0'), 0);
    assert_eq!(ascii_char_to_num(b'a'), 10);
    assert_eq!(ascii_char_to_num(b'F'), 15);
}


#[test]
fn test_macro() {
    assert_eq!(&hex!("010aff"), &[1, 10, 255]);
    // lower-case letters
    assert_eq!(&hex!("abcd"), &[0xab, 0xcd]);
    // upper-case letters
    assert_eq!(&hex!("ABCD"), &[0xab, 0xcd]);
    // mixed-case letters
    assert_eq!(&hex!("AbcD"), &[0xab, 0xcd]);
}
