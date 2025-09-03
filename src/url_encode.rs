//! URL encoding utilities for no_std compatibility.
//!
//! This module provides URL encoding functionality that works in both std and no_std environments.
//! The encoding follows RFC 3986 standards for percent-encoding.

use core::fmt::Write;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// URL encode a string according to RFC 3986.
///
/// This function percent-encodes all characters except unreserved characters
/// (ALPHA / DIGIT / "-" / "." / "_" / "~"). Spaces are encoded as '+' for
/// form-encoded data compatibility.
///
/// # Arguments
///
/// * `input` - The string to be URL encoded
///
/// # Returns
///
/// A new `String` containing the URL-encoded version of the input.
///
/// # Examples
///
/// ```
/// use bug::url_encode::encode;
///
/// // Basic encoding
/// assert_eq!(encode("hello world"), "hello+world");
/// 
/// // Special characters
/// assert_eq!(encode("hello@world.com"), "hello%40world.com");
/// 
/// // Unreserved characters remain unchanged
/// assert_eq!(encode("hello-world_123.txt~"), "hello-world_123.txt~");
/// 
/// // Unicode characters
/// assert_eq!(encode("café"), "caf%C3%A9");
/// ```
pub fn encode(input: &str) -> String {
    let mut output = String::new();
    
    for byte in input.bytes() {
        match byte {
            // Unreserved characters (ALPHA / DIGIT / "-" / "." / "_" / "~")
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                output.push(byte as char);
            }
            // Space encoded as +
            b' ' => {
                output.push('+');
            }
            // Everything else percent-encoded
            _ => {
                write!(&mut output, "%{:02X}", byte).unwrap();
            }
        }
    }
    
    output
}