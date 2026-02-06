use rhai::plugin::*;

#[export_module]
pub mod base64_rhai {
    use base64::{prelude::BASE64_STANDARD, Engine};

    #[rhai_fn(return_raw)]
    pub fn decode(text: &str) -> Result<String, Box<rhai::EvalAltResult>> {
        BASE64_STANDARD
            .decode(text)
            .map_err(|e| e.to_string().into())
            .and_then(|bytes| {
                std::str::from_utf8(&bytes)
                    .map_err(|e| e.to_string().into())
                    .map(|s| s.to_owned())
            })
    }
}

#[cfg(test)]
mod tests {
    use rhai::EvalAltResult;

    use super::base64_rhai;

    #[test]
    pub fn test_decode_ok() {
        let encoded = "UXVhbnR1bSBwaHlzaWNzIG1ha2VzIG1lIHNvIGhhcHB5LiBJdCdzIGxpa2UgbG9va2luZyBhdCB0aGUgdW5pdmVyc2UgbmFrZWQu";
        let decoded = base64_rhai::decode(encoded);

        assert_eq!(
            "Quantum physics makes me so happy. It's like looking at the universe naked.",
            decoded.unwrap()
        );
    }

    #[test]
    pub fn test_decode_err_on_invalid_input() {
        let encoded = "PupsbärchenSonderzeichen"; // Definitely not valid base64
        let decoded = base64_rhai::decode(encoded);

        let error = decoded.unwrap_err();
        assert!(matches!(*error, EvalAltResult::ErrorRuntime(..)));
    }

    #[test]
    pub fn test_decode_err_on_non_utf8_result() {
        let encoded = "76VVtlXhsXI="; // Just 8 random bytes that are definitely not UTF-8
        let decoded = base64_rhai::decode(encoded);

        let error = decoded.unwrap_err();
        assert!(matches!(*error, EvalAltResult::ErrorRuntime(..)));
    }
}
