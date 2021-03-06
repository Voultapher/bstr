/*!
An experimental byte string library.

Byte strings are just like standard Unicode strings with one very important
difference: byte strings are only *conventionally* UTF-8 while Rust's standard
Unicode strings are *guaranteed* to be valid UTF-8. The primary motivation for
this type is for handling arbitrary bytes that are mostly UTF-8.

# Overview

There are two primary types in this crate:

* [`BString`](struct.BString.html) is an owned growable byte string buffer,
  analogous to `String`.
* [`BStr`](struct.BStr.html) is a byte string slice, analogous to `str`.

Additionally, the free function [`B`](fn.B.html) serves as a convenient short
hand for writing byte string literals.

# Quick examples

Byte strings are effectively the same thing as a `Vec<u8>` or a `&[u8]`, except
they provide a string oriented API. Operations such as iterating over
graphemes, searching for substrings, replacing substrings, trimming and case
conversion are examples of things not provided on the standard `&[u8]` APIs
but are provided by this crate. For example, this code iterates over all of
occurrences of a subtring:

```
use bstr::B;

let s = B("foo bar foo foo quux foo");

let mut matches = vec![];
for start in s.find_iter("foo") {
    matches.push(start);
}
assert_eq!(matches, [0, 8, 12, 21]);
```

Here's another example showing how to do a search and replace:

```
use bstr::B;

let old = B("foo bar foo foo quux foo");
let new = old.replace("foo", "hello");
assert_eq!(new, "hello bar hello hello quux hello");
```

And here's an example that shows case conversion, even in the presence of
invalid UTF-8:

```
use bstr::{B, BString};

let mut lower = BString::from("hello β");
lower[0] = b'\xFF';
// lowercase β is uppercased to Β
assert_eq!(lower.to_uppercase(), B(b"\xFFELLO \xCE\x92"));
```

# When should I use byte strings?

This library is somewhat of an experiment that reflects my hypothesis that
UTF-8 by convention is a better trade off in some circumstances than guaranteed
UTF-8. It's possible, perhaps even likely, that this is a niche concern for
folks working closely with core text primitives.

The first time this idea hit me was in the implementation of Rust's regex
engine. In particular, very little of the internal implementation cares at all
about searching valid UTF-8 encoded strings. Indeed, internally, the
implementation converts `&str` from the API to `&[u8]` fairly quickly and
just deals with raw bytes. UTF-8 match boundaries are then guaranteed by the
finite state machine itself rather than any specific string type. This makes it
possible to not only run regexes on `&str` values, but also on `&[u8]` values.

Why would you ever want to run a regex on a `&[u8]` though? Well, `&[u8]` is
the fundamental way at which one reads data from all sorts of streams, via the
standard library's [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
trait. In particular, there is no platform independent way to determine whether
what you're reading from is some binary file or a human readable text file.
Therefore, if you're writing a program to search files, you probably need to
deal with `&[u8]` directly unless you're okay with first converting it to a
`&str` and dropping any bytes that aren't valid UTF-8. (Or otherwise determine
the encoding---which is often impractical---and perform a transcoding step.)
Often, the simplest and most robust way to approach this is to simply treat the
contents of a file as if it were mostly valid UTF-8 and pass through invalid
UTF-8 untouched. This may not be the most correct approach though!

One case in particular exacerbates these issues, and that's memory mapping
a file. When you memory map a file, that file may be gigabytes big, but all
you get is a `&[u8]`. Converting that to a `&str` all in one go is generally
not a good idea because of the costs associated with doing so, and also
because it generally causes one to do two passes over the data instead of
one, which is quite undesirable. It is of course usually possible to do it an
incremental way by only parsing chunks at a time, but this is often complex to
do or impractical. For example, many regex engines only accept one contiguous
sequence of bytes at a time with no way to perform incremental matching.

In summary, the conventional UTF-8 byte strings provided by this library is an
experiment. They are definitely useful in some limited circumstances, but how
useful they are more broadly isn't clear yet.

# `bstr` in public APIs

Since this library is still experimental, you should not use it in the public
API of your crates until it hits `1.0` (unless you're OK with with tracking
breaking releases of `bstr`). It is a priority to move this crate to `1.0`
expediently so that `BString` and `BStr` may be used in the public APIs of
other crates. While both `BString` and `BStr` do provide zero cost ways of
converting between `Vec<u8>` and `&[u8]`, it is often convenient to provide
trait implementations for `BString` and `BStr`, which requires making `bstr` a
public dependency.

# Differences with standard strings

The primary difference between `BStr` and `str` is that the former is
conventionally UTF-8 while the latter is guaranteed to be UTF-8. The phrase
"conventionally UTF-8" means that a `BStr` may contain bytes that do not form
a valid UTF-8 sequence, but operations defined on the type are generally most
useful on valid UTF-8 sequences. For example, iterating over Unicode codepoints
or grapheme clusters is an operation that is only defined on valid UTF-8.
Therefore, when invalid UTF-8 is encountered, the Unicode replacement codepoint
is substituted. Thus, a byte string that is not UTF-8 at all is of limited
utility when using these methods.

However, not all operations on byte strings are specifically Unicode aware. For
example, substring search has no specific Unicode semantics ascribed to it. It
works just as well for byte strings that are completely valid UTF-8 as for byte
strings that contain no valid UTF-8 at all. Similarly for replacements and
various other operations.

Aside from the difference in how UTF-8 is handled, the APIs between `BStr` and
`str` (and `BString` and `String`) are intentionally very similar, including
maintaining the same behavior for corner cases in things like substring
splitting. There are, however, some differences:

* Substring search is not done with `matches`, but instead, `find_iter`.
  In general, this crate does not define any generic
  [`Pattern`](https://doc.rust-lang.org/std/str/pattern/trait.Pattern.html)
  infrastructure, and instead prefers adding new methods for different
  argument types. For example, `matches` can search by a `char` or a `&str`,
  where as `find_iter` can only search by a byte string. `find_char` can be
  used for searching by a `char`.
* Since `SliceConcatExt` in the standard library is unstable, it is not
  possible to reuse that to implement `join` and `concat` methods. Instead,
  [`join`](fn.join.html) and [`concat`](fn.concat.html) are provided as free
  functions that perform a similar task.
* This library bundles in a few more Unicode operations, such as grapheme,
  word and sentence iterators. More operations, such as normalization and
  case folding, may be provided in the future.
* Some `String`/`str` APIs will panic if a particular index was not on a valid
  UTF-8 code unit sequence boundary. Conversely, no such checking is performed
  in this crate, as is consistent with treating byte strings as a sequence of
  bytes. This means callers are responsible for maintaining a UTF-8 invariant
  if that's important.

Otherwise, you should find most of the APIs between this crate and the standard
library to be very similar, if not identical.

# Handling of invalid UTF-8

Since byte strings are only *conventionally* UTF-8, there is no guarantee
that byte strings contain valid UTF-8. Indeed, it is perfectly legal for a
byte string to contain arbitrary bytes. However, since this library defines
a *string* type, it provides many operations specified by Unicode. These
operations are typically only defined over codepoints, and thus have no real
meaning on bytes that are invalid UTF-8 because they do not map to a particular
codepoint.

For this reason, whenever operations defined only on codepoints are used, this
library will automatically convert invalid UTF-8 to the Unicode replacement
codepoint, `U+FFFD`, which looks like this: `�`. For example, an
[iterator over codepoints](struct.Chars.html) will yield a Unicode
replacement codepoint whenever it comes across bytes that are not valid UTF-8:

```
use bstr::B;

let bs = B(b"a\xFF\xFFz");
let chars: Vec<char> = bs.chars().collect();
assert_eq!(vec!['a', '\u{FFFD}', '\u{FFFD}', 'z'], chars);
```

There are a few ways in which invalid bytes can be substituted with a Unicode
replacement codepoint. One way, not used by this crate, is to replace every
individual invalid byte with a single replacement codepoint. In contrast, the
approach this crate uses is called the "substitution of maximal subparts," as
specified by the Unicode Standard (Chapter 3, Section 9). (This approach is
also used by [W3C's Encoding Standard](https://www.w3.org/TR/encoding/).) In
this strategy, a replacement codepoint is inserted whenever a byte is found
that cannot possibly lead to a valid UTF-8 code unit sequence. If there were
previous bytes that represented a *prefix* of a well-formed UTF-8 code unit
sequence, then all of those bytes (up to 3) are substituted with a single
replacement codepoint. For example:

```
use bstr::B;

let bs = B(b"a\xF0\x9F\x87z");
let chars: Vec<char> = bs.chars().collect();
// The bytes \xF0\x9F\x87 could lead to a valid UTF-8 sequence, but 3 of them
// on their own are invalid. Only one replacement codepoint is substituted,
// which demonstrates the "substitution of maximal subparts" strategy.
assert_eq!(vec!['a', '\u{FFFD}', 'z'], chars);
```

If you do need to access the raw bytes for some reason in an iterator like
`Chars`, then you should use the iterator's "indices" variant, which gives
the byte offsets containing the invalid UTF-8 bytes that were substituted with
the replacement codepoint. For example:

```
use bstr::{B, BStr};

let bs = B(b"a\xE2\x98z");
let chars: Vec<(usize, usize, char)> = bs.char_indices().collect();
// Even though the replacement codepoint is encoded as 3 bytes itself, the
// byte range given here is only two bytes, corresponding to the original
// raw bytes.
assert_eq!(vec![(0, 1, 'a'), (1, 3, '\u{FFFD}'), (3, 4, 'z')], chars);

// Thus, getting the original raw bytes is as simple as slicing the original
// byte string:
let chars: Vec<&BStr> = bs.char_indices().map(|(s, e, _)| &bs[s..e]).collect();
assert_eq!(vec![B("a"), B(b"\xE2\x98"), B("z")], chars);
```
*/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "unicode")]
#[macro_use]
extern crate lazy_static;
extern crate memchr;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(feature = "unicode")]
extern crate regex_automata;
#[cfg(feature = "serde1-nostd")]
extern crate serde;
#[cfg(test)]
extern crate ucd_parse;

pub use bstr::{
    B, BStr,
    Bytes,
    Finder, FinderReverse, Find, FindReverse,
    Split, SplitReverse, SplitN, SplitNReverse,
    Fields, FieldsWith,
    Lines, LinesWithTerminator,
};
#[cfg(feature = "std")]
pub use bstring::{BString, DrainBytes, FromUtf8Error, concat, join};
pub use slice_index::SliceIndex;
#[cfg(feature = "unicode")]
pub use unicode::{
    Graphemes, GraphemeIndices,
    Sentences, SentenceIndices,
    Words, WordIndices, WordsWithBreaks, WordsWithBreakIndices,
};
pub use utf8::{
    Utf8Error, Chars, CharIndices,
    decode as decode_utf8,
    decode_last as decode_last_utf8,
};

mod ascii;
mod bstr;
#[cfg(feature = "std")]
mod bstring;
mod cow;
mod impls;
#[cfg(feature = "std")]
pub mod io;
mod search;
mod slice_index;
#[cfg(test)]
mod tests;
#[cfg(feature = "unicode")]
mod unicode;
mod utf8;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn oibits() {
        use std::panic::{RefUnwindSafe, UnwindSafe};

        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_unwind_safe<T: RefUnwindSafe + UnwindSafe>() {}

        assert_send::<&BStr>();
        assert_sync::<&BStr>();
        assert_unwind_safe::<&BStr>();
        assert_send::<BString>();
        assert_sync::<BString>();
        assert_unwind_safe::<BString>();

        assert_send::<Finder>();
        assert_sync::<Finder>();
        assert_unwind_safe::<Finder>();
        assert_send::<FinderReverse>();
        assert_sync::<FinderReverse>();
        assert_unwind_safe::<FinderReverse>();
    }
}
