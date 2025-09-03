use core::fmt::Write;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// URL encode a string for no_std compatibility
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