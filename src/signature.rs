use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Verifies a GitHub `X-Hub-Signature-256` header against the raw request
/// body. Comparison uses subtle's constant-time equality; length mismatch
/// returns false immediately, which is safe here because the expected HMAC
/// length is fixed and public.
pub fn verify_signature(secret: &str, body: &[u8], signature_header: &str) -> bool {
    let Some(hex_signature) = signature_header.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(claimed) = hex::decode(hex_signature) else {
        return false;
    };
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(body);
    let computed = mac.finalize().into_bytes();
    computed.as_slice().ct_eq(claimed.as_slice()).into()
}

/// Computes the `sha256=<hex>` signature GitHub would send for this body.
/// Lives outside `#[cfg(test)]` so integration tests can build valid requests.
pub fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-answer test: HMAC-SHA256("It's a Secret to Everybody", "Hello, World!")
    // from GitHub's own webhook documentation example.
    #[test]
    fn accepts_githubs_documented_example() {
        let header = "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";
        assert!(verify_signature(
            "It's a Secret to Everybody",
            b"Hello, World!",
            header
        ));
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let sig = sign("test-secret", b"{\"a\":1}");
        assert!(sig.starts_with("sha256="));
        assert!(verify_signature("test-secret", b"{\"a\":1}", &sig));
    }

    #[test]
    fn rejects_wrong_secret() {
        let sig = sign("right-secret", b"payload");
        assert!(!verify_signature("wrong-secret", b"payload", &sig));
    }

    #[test]
    fn rejects_tampered_body() {
        let sig = sign("test-secret", b"payload");
        assert!(!verify_signature("test-secret", b"payload-tampered", &sig));
    }

    #[test]
    fn rejects_malformed_headers() {
        for header in ["", "sha1=abc", "sha256=", "sha256=zzzz", "757107ea"] {
            assert!(
                !verify_signature("test-secret", b"payload", header),
                "accepted: {header}"
            );
        }
    }
}
