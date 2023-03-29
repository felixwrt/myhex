
I've been using the `hex-literal` crate in several projects when working with hex strings and enjoyed it a lot. It allows me to write binary data as a hex string and transforms these strings into byte arrays at compile time. For example, using `hex_literal::hex!("010aff")` is equivalent to writing `[0x01, 0x0a, 0xff]`. For larger data, using `hex!` is much more compact and improves readability a lot.

The `hex-literal` crate is implemented using [procedural macros](TODO) and I was interested to see whether something similar could be implemented using const fn / const generics in Rust. 

## The task

The task is pretty simple. We want to have a function `hex()` that takes a string as input and turns it into a sequence of bytes. Each pair of characters in the input string (e.g. `"0a"`) turn into one byte in the output (`10`). The input needs to have an even number of characters. For simplicity, we'll also require that the input string may only contain the characters `0-9`, `A-F` and `a-f`. Other characters like whitespace or separators are not allowed.

Additionally, we want to be able to use our function in `const` contexts like in `const MY_BYTES = hex("12ab89");`.

## Regular (non-const) implementation

Let's start simple by first implementing a regular (non-const) function that does the transformation.

```rust
/// Turns a hex string into a vector of bytes.
pub fn hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    let output_len = bytes.len() / 2;

    // TODO: create vector of length `output_len` and fill it 
    // with the bytes encoded in `bytes`.
    todo!()
}
```

All valid characters in the input string are ASCII characters and therefore only take one byte in the input string. Because of that, we can convert the `&str` into a byte slice `&[u8]` and work with that instead. This simplifies the implementation (and probably also improves performance). 

We don't bother doing proper error handling using `Result`s and such but simply panic when the input is invalid. You wouldn't normally do this in a library, but it's ok for this example and we'll talk about error handling again when making everything `const`. 

To do the actual decoding of the input string, we need a way to convert a single character into the number it represents. For example, the character `b'F'` would be turned into the number `15`. Fortunately, Rusts powerful `match` expressions make this easy:

```rust
/// Turns a single ascii character into the number it represents.
fn ascii_char_to_num(ascii_char: u8) -> u8 {
    match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'f' => ascii_char - b'a' + 10,
        b'A'..=b'F' => ascii_char - b'A' + 10,
        _ => panic!("Invalid character"),
    }
}
```

We match the character to one of three groups (numbers, lower- and upper-case letters), calculate the offset from the first character in that group and add the number the first character in the group represents to the offset. This works because numbers and letters are encoded in order in ASCII.

Now that the decoding of a single character is done, we can go back to the implementation of `hex()`:

```rust
/// Turns a hex string into a vector of bytes.
pub fn hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    let output_len = bytes.len() / 2;

    let mut vec = Vec::with_capacity(output_len);
    for idx in 0..output_len {
        let msb = ascii_char_to_num(bytes[idx * 2]);
        let lsb = ascii_char_to_num(bytes[idx * 2 + 1]);
        vec.push((msb<<4) + lsb);
    }
    vec
}
```

We'll first create a vector with a capacity of half the input length. We already know that the output length is half of the input length (as each bytes is encoded using two characters), so using `Vec::with_capacity()` prevents unnecessary allocations while filling the vector.

Then, we decode each byte in a loop: We first read two bytes from the input and decode them into `msb` and `lsb`, where `msb` contains the upper 4 bits of the byte and `lsb` contains the lower 4 bits of the byte. Afterwards we calculate the byte using `(msb << 4) + lsb` and push it to the output vector.

And that's that! You can check out the full code [here][TODO!]. Now on to the interesting part:

## About `const`

Let's try to use `hex()` in a constant:

```rust
const MY_BYTES: : Vec<u8> = hex("1234abcdef");
```

Unfortunately, using `cargo build` shows that something's wrong:

```
error[E0015]: cannot call non-const fn `hex` in constants
 --> src/lib.rs:1:27
  |
1 | const MY_BYTES: Vec<u8> = hex("1234abcdef");
  |                           ^^^^^^^^^^^^^^^^^
  |
  = note: calls in constants are limited to constant functions, tuple structs and tuple variants
```

The compiler tells us that we can't use `hex()` in a constant and that we'd need to use a `const fn`. Why's that?

Up to now, our `hex()` function is just a regular Rust function. During compilation, it is turned into machine code and is executed once we run the resulting binary. Constants are different in that they are evaluated at compile-time and only the result of the evaluation is put into the binary. At runtime, the constant value is simply read from the correct location in the executable. 

So when using `hex()` in a constant, the compiler needs to be able to evaluate the function at compile-time. The Rust compiler cannot evaluate arbitrary functions at compile time and even though more and more Rust features are available, only a subset of Rust can be evaluated at compile-time. For example, allocations cannot be evaluated at compile-time currently. 

Alright, so some functions can be evaluated at compile time while others can't. In Rust, that distinction is made explicit by using the `const` keyword. Adding it to a function declaration (e.g. `const fn my_function() { ... }`) denotes that the function can be evaluated at compile time and therefore can be used in contexts where a constant value is required. On the other hand, it also means that the function may only use a subset of Rust. If the function uses a feature that isn't supported in `const fn`s, the compiler will emit an error.

Let's take a look at a simple example:

```rust
fn add(a: usize, b: usize) -> usize {
    a + b
}

const SUM: usize = add(11, 31);
```

As before, this leads to a compiler error:

```
error[E0015]: cannot call non-const fn `add` in constants
 --> src/lib.rs:5:20
  |
5 | const SUM: usize = add(11, 31);
  |                    ^^^^^^^^^^^
  |
  = note: calls in constants are limited to constant functions, tuple structs and tuple variants
```

Changing `add()` into a `const fn` does the trick:

```
   Compiling myhex v0.1.0 (/home/felix/pj/myhex)
    Finished dev [unoptimized + debuginfo] target(s) in 0.10s
```

## Making `hex()` `const`

Before trying to convert `hex()` into a `const fn`, let's first do it with `ascii_char_to_num` by simply adding `const` in front of the `fn` keyword:

```rust
/// Turns a single ascii character into the number it represents.
const fn ascii_char_to_num(ascii_char: u8) -> u8 {
    match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'f' => ascii_char - b'a' + 10,
        b'A'..=b'F' => ascii_char - b'A' + 10,
        _ => panic!("Invalid character"),
    }
}
```

See if it still compiles:

```
   Compiling myhex v0.1.0 (/home/felix/pj/myhex)
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
```

Yey, that's our first `const fn`! But wait, how is that `panic!` going to work when the function is evaluated at compile time? Is this going to make the compiler panic? Let's to a quick test:

```rust
fn main() {
    let num = ascii_char_to_num(b'X');
    println!("{num}")
}
```

`cargo build`:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
```

Interesting, no errors during compilation.

`cargo run`:

```
thread 'main' panicked at 'Invalid character', src/main.rs:39:14
```

Seems like we get an error at runtime, just like if we had used a regular function instead of `const fn`.

What happened here is that the compiler was treating `ascii_char_to_num` like it was a regular function. It compiled it and it was only executed when we ran the program using `cargo run`. 

This is an important point: just because a function is `const` doesn't mean that it will always be evaluated at compile time. `const` functions are a subset of regular Rust functions and can also be compiled and executed as such.

If we want to be sure that the function is evaluated at compile-time, we can force the compiler to do it by storing the result in a `const` variable:

```rust
fn main() {
    const num: u8 = ascii_char_to_num(b'X');
    println!("{num}")
}
```

Using `cargo build`:

```
error[E0080]: evaluation of constant value failed
  --> src/main.rs:39:14
   |
39 |         _ => panic!("Invalid character"),
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |              |
   |              the evaluated program panicked at 'Invalid character', src/main.rs:39:14
   |              inside `ascii_char_to_num` at /home/felix/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/panic.rs:57:9
...
44 |     const num: u8 = ascii_char_to_num(b'X');
   |                     ----------------------- inside `num` at src/main.rs:44:21
   |
```

Now that's what we were looking for. The compiler is evaluating our function and when it encounters the `panic!`, it prints the error and aborts compilation. Using constant evaluation and `const fn`, we can turn runtime errors into compile-time errors, neat!

Alright, back to our `ascii_char_to_num()` function:

```rust
/// Turns a single ascii character into the number it represents.
const fn ascii_char_to_num(ascii_char: u8) -> u8 {
    match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'f' => ascii_char - b'a' + 10,
        b'A'..=b'F' => ascii_char - b'A' + 10,
        _ => panic!("Invalid character"),
    }
}
```

Normally, you'd use `Result` or `Option` in a function like this, which would allow users of it to handle the error case without crashing the whole program. But since we're only planning to use this function at compile-time, panicing means that if a user uses `hex!()` with an invalid character, they will immediately get a compilation error. That's great as the check is done before runtime and once your code compiles, you can be sure that the strings used in `hex!()` are correct and that there won't be any runtime panics from it.

Speaking of `hex!()`, let's go ahead and make it `const` as well:

```rust
/// Turns a hex string into a vector of bytes.
pub const fn hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    let output_len = bytes.len() / 2;

    let mut vec = Vec::with_capacity(output_len);
    for idx in 0..output_len {
        let msb = ascii_char_to_num(bytes[idx * 2]);
        let lsb = ascii_char_to_num(bytes[idx * 2 + 1]);
        vec.push((msb<<4) + lsb);
    }
    vec
}
```

This fails to compile with the following error:

```
error[E0658]: `for` is not allowed in a `const fn`
  --> src/main.rs:18:5
   |
18 | /     for idx in 0..output_len {
19 | |         let msb = ascii_char_to_num(bytes[idx * 2]);
20 | |         let lsb = ascii_char_to_num(bytes[idx * 2 + 1]);
21 | |         vec.push((msb<<4) + lsb);
22 | |     }
   | |_____^
   |
   = note: see issue #87575 <https://github.com/rust-lang/rust/issues/87575> for more information
```

Note that this is only the tip of the iceberg though and fixing the `for` loop will yield more compilation errors.

In a situation like this, there's two approaches: The first is to make changes to the implementation until the compiler is happy, the second is to reduce the implementation until it compiles and successively adding more parts afterwards. For this post, I've chosen the second approach. With a bit of prior knowledge on my side, we can get to the following (partial) implementation that compiles:

```rust
pub const fn hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");

    let mut vec = Vec::new();
    vec
}
```

That's interesting, isn't it? I've previously claimed that allocations aren't possible in `const fn`s and yet, I'm creating a new vector and it works just fine. The reason why it works is that `Vec::new` doesn't actually allocate any heap memory. An allocation is only performed once the first element is inserted into the `Vec`. Looking into the standard library documentation, we can see that `Vec::new` is `const` while `Vec::push` (and many others) aren't. While the code above compiles, `Vec` is not really useful in `const fn`, so we'll better use something else. Knowing that allocations aren't possible leaves us with the statically-sized counterpart to vectors, the `array`. The downside of an array is that we need to provide the size of it. Since we don't want to hard-code the size within `hex()`, we'll make it a generic parameter (which is possible thanks to const generics).

```rust
pub const fn hex<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");

    let mut arr = [0; N];
    arr
}
```

Using this implementation, we need to make sure that we call `hex()` with the correct value of `N`:

```rust
fn main() {
    const STR: &str = "abcd1234";
    const MY_BYTES: [u8; STR.len() / 2] = hex(STR);
}
```

That's not as nice at it could be, but it's the best we can do in stable Rust today. There's a way to make the usage more ergonomic which I'll share further below. For now, let's just add an assertion in `hex()` to make sure that the provided value `N` is in fact half of the input string's length:

```rust
pub const fn hex<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();

    assert!(bytes.len() % 2 == 0, "Length needs to be even");
    assert!(bytes.len() == N * 2, "Invalid length (`N * 2 == s.len()` not satisfied).");
    
    let mut arr = [0; N];
    arr
}
```

The first assertion might seem redundant as the second one also checks that `bytes.len()` is even, but there's a reason for keeping both. We want to first chech for an input length that's invalid, as it's likely the more common error case and also one that's easier to explain to a user. If the input length is valid but the `N` parameter isn't set correctly, we will still report that as an error.

Continuing with the implementation of `hex()`, let's add the loop back. We know that `for` isn't allowed, so let's use the next best thing, which is `while`:

```rust
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
```

`cargo build` shows us that our code still compiles. 

Let's write a quick test to make sure our implementation still works:

```rust
#[test]
fn test_basic() {
    const STR: &str = "abcd1234";
    const MY_BYTES: [u8; STR.len() / 2] = hex(STR);
    assert_eq!(MY_BYTES, [0xab, 0xcd, 0x12, 0x34]);
}
```

<TODO OUTPUT>


Yup, still works!

## Making it nice

We're now at a point where we can create byte arrays from hex strings at compile time, but it certainly doesn't look nice. Take a look at the following example, where we're creating a constant using `hex()`:

```rust
fn main() {
    const STR: &str = "abcd1234";
    const MY_BYTES: [u8; STR.len() / 2] = hex(STR);
}
```

If we don't need the byte array to be constant, we can also bind the output of `hex()` to a variable instead:

```rust
fn main() {
    const STR: &str = "1111";
    let my_bytes = hex::<{STR.len() / 2}>(STR);
}
```

But again, be carful. The code above isn't guaranteed to be evaluated at compile time and could end up being a regular function call at runtime. If we want to make sure that the evaluation happens at compile-time, we can pass the result of `hex()` to a constant first and assign that to our variable. We can also do that in a local scope such that we're only exposing a single variable to the rest of our program:

```rust
fn main() {
    let my_bytes = {
        const STR: &str = "1111";
        const X: [u8; STR.len() / 2] = hex(STR);
        X
    };
}
```

Unfortunately, this little trick doesn't work when the result should be a constant. We would need to provide the constant's type outside of the scope in which `STR` is defined. As we need to access `STR.len()` to calculate the length of the array, this doesn't work. The only way around it would be do duplicate the string literal, like so:

```rust
fn main() {
    const MY_BYTES: [u8; "abcd1234".len() / 2] = hex("abcd1234");
}
```

For hand-written code, that's a terrible idea. As soon as someone updates one of the strings without updating the other, you'll get errors. At least, we know that this would be a compilation errors instead of a runtime panics, so it's not as bad as it could be.

Let's imagine we're writing a program that uses multiple constants and variables built from hex strings:

```rust
fn main() {
    const MY_BYTES: [u8; "abcd1234".len() / 2] = hex("abcd1234");

    const MY_BYTES_2: [u8; "9876fedc".len() / 2] = hex("9876fedc");

    let my_bytes_3 = {
        const STR: &str = "1111";
        const X: [u8; STR.len() / 2] = hex(STR);
        X
    };

    let my_bytes_4 = {
        const STR: &str = "2222";
        const X: [u8; STR.len() / 2] = hex(STR);
        X
    };
}
```

Looking at the invocations of `hex()`, we can see two patterns, one for creating constants and one for creating variables. I've used `SHOUT_CASE` for things that would be different for different cases.

```
// for constants
const CONST_NAME: [u8; STRING_LITERAL.len() / 2] = hex(STRING_LITERAL);

// for variables
let VARIABLE_NAME = {
    const STR: &str = STRING_LITERAL;
    const X: [u8; STR.len() / 2] = hex(STR);
    X
};
```

Once you've identified patterns like these that you expect to be used a lot and that can't be expressed with functions, traits etc., you've probably found a good use case for macros.

## Writing a macro

Rust comes with two kinds of macros: declarative ("macros by example") and procedural macros. Procedural macros are definitely the more powerful ones, allowing you to write regular Rust code that transforms the macro input to some output. The downside of them is an increase in compilation times and that they are even more challenging for code intelligence tools like `rust-analyzer`. As a rule of thumb, you should prefer declarative macros if they can express your use case.

Declarative macros are written by describing how the input is expected to look like (using placeholders for user-provided elements) and how the corresponding output looks like. I'm not going to give a full explanation of declarative macros here, but will instead show how they can be used to make the usage of `hex()` more ergonomic.

Let's take a look at the case of creating a constant first. Wouldn't it be nice if we had a `hex!()` macro that allowed us to write something like this:

```rust
hex!{
    const MY_BYTES = "abcd";
}
```

Going back to the pseudo-notation I used earlier, we want our macro to turn input like this

```
const CONST_NAME = STRING_LITERAL;
```

into an output like this:

```
const CONST_NAME: [u8; STRING_LITERAL.len() / 2] = hex(STRING_LITERAL);
```

Without further ado, here's the macro that does it:

```rust
macro_rules! hex {
    (const $name:ident = $hex:expr;) => {
        const $name: [u8; $hex.len() / 2] = $crate::hex!($hex);
    };
}
```

Doesn't look too difficult, does it? The placeholders look different and there's a `match` like syntax to separate the input and output patterns, but that's basically it.

Let's see if it works:

```rust
hex!{
    const MY_BYTES = "abcd1234";
}
assert_eq!(MY_BYTES, [0xab, 0xcd, 0x12, 0x34]);
```

Works great, let's move on to the other case! We could do the same as in the constant case above, which would mean using the macro like this:

```rust
hex!{
    let my_bytes = "abcd1234";
}
```

That'd certainly be an option, but isn't needed here. Let's take another look at the pattern we created earlier:

```
let VARIABLE_NAME = {
    const STR: &str = STRING_LITERAL;
    const X: [u8; STR.len() / 2] = hex(STR);
    X
};
```

Looking closely, we can see that the macro doesn't need to span the entire declaration, but could be reduced to the right-hand side of it instead. In this case, using the macro would look very similar to calling the `hex()` function:

```rust
let my_bytes = hex!("abcd1234");
```

Writing the corresponding macro for this is quite easy:

```rust
macro_rules! hex {
    ($hex:expr) => {
        {
            const STR: &'static str = $hex;
            const X: [u8; {STR.len() / 2}] = $crate::hex(STR);
            X
        }
    };
}
```

```rust
let my_bytes = hex!("abcd1234");
assert_eq!(my_bytes, [0xab, 0xcd, 0x12, 0x34]);
```

Works just as expected! 

You might be wondering whether we need two macros for these two use cases. As it turns out, macros in Rust support multiple patterns and just like with `match` the first pattern that matches the input is used. Combining both patterns, we get our final `hex!()` macro:

```rust
macro_rules! hex {
    (const $name:ident = $hex:expr;) => {
        const $name: [u8; $hex.len() / 2] = $crate::hex!($hex);
    };
    ($hex:expr) => {
        {
            const STR: &'static str = $hex;
            const X: [u8; {STR.len() / 2}] = $crate::hex(STR);
            X
        }
    };
}
```

Let's see how using the macro changes the larger example program we looked at earlier. Here's the example again:

```rust
fn main() {
    const MY_BYTES: [u8; "abcd1234".len() / 2] = hex("abcd1234");

    const MY_BYTES_2: [u8; "9876fedc".len() / 2] = hex("9876fedc");

    let my_bytes_3 = {
        const STR: &str = "1111";
        const X: [u8; STR.len() / 2] = hex(STR);
        X
    };

    let my_bytes_4 = {
        const STR: &str = "2222";
        const X: [u8; STR.len() / 2] = hex(STR);
        X
    };
}
```

Using our `hex!()` macro, we can turn it into this:

```rust
fn main() {
    hex!{
        const MY_BYTES = "abcd1234";
    }
    
    hex!{
        const MY_BYTES_2 = "9876fedc";
    }
    
    let my_bytes_3 = hex!("1111");

    let my_bytes_4 = hex!("2222");
}
```

I'm quite happy with the result. 

## Summary

You've made it! I naively set out to write a quick blog post about implementing a simplified `const fn` version of `hex-literal`, but as you've seen, it turned out to be quite a journey. Along the way, we learned about ASCII encoding, constant evaluation, `const fn` and even declarative macros.

I'm of course not the first one to use `const fn` and declarative macros to implement compile-time hex string to byte array conversions. Take a look at the [hexlit](https://lib.rs/crates/hexlit) and [hex_lit](https://lib.rs/crates/hex_lit) crates if you want to use something like this in your own codebase. Not to mention [hex-literal](https://lib.rs/crates/hex-literal) which is based on procedural macros and can be used with quite old versions of Rust.

Discussion on /r/rust.