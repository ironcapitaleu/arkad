//! # Test Signing Key
//!
//! Provides [`TestSigningKey`], a key pair that a test generates to sign login tokens.

use aws_lc_rs::encoding::AsDer;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::rsa::{KeyPair as RsaKeyPair, KeySize};
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::{Value, json};

/// A key pair that a test generates, with the public half as a JWK.
///
/// Each call generates a new key, so two keys with the same key ID still differ.
pub struct TestSigningKey {
    kid: String,
    algorithm: Algorithm,
    encoding_key: EncodingKey,
    jwk: Jwk,
}

impl TestSigningKey {
    /// Generates an `ES256` key pair on the P-256 curve.
    ///
    /// # Panics
    ///
    /// Panics if the system cannot generate a key.
    #[must_use]
    pub fn es256(kid: &str) -> Self {
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &SystemRandom::new())
                .expect(
                    "Given the system random source, generating a P-256 key should always succeed",
                );
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
            .expect("Given a freshly generated PKCS#8 document, parsing it should always succeed");
        // An uncompressed P-256 point is `0x04 || x || y`, with 32 bytes for each coordinate.
        let point = key_pair.public_key().as_ref();
        let jwk = json!({
            "kty": "EC",
            "crv": "P-256",
            "x": URL_SAFE_NO_PAD.encode(&point[1..33]),
            "y": URL_SAFE_NO_PAD.encode(&point[33..65]),
            "kid": kid,
            "alg": "ES256",
            "use": "sig",
        });
        Self::new(
            kid,
            Algorithm::ES256,
            EncodingKey::from_ec_der(pkcs8.as_ref()),
            &jwk,
        )
    }

    /// Generates an `RS256` key pair with a 2048-bit modulus.
    ///
    /// # Panics
    ///
    /// Panics if the system cannot generate a key.
    #[must_use]
    pub fn rs256(kid: &str) -> Self {
        let key_pair = RsaKeyPair::generate(KeySize::Rsa2048)
            .expect("Given the system random source, generating an RSA key should always succeed");
        let der = key_pair
            .as_der()
            .expect("Given a freshly generated RSA key, encoding it should always succeed");
        let public_key = key_pair.public_key();
        let jwk = json!({
            "kty": "RSA",
            "n": URL_SAFE_NO_PAD.encode(public_key.modulus().big_endian_without_leading_zero()),
            "e": URL_SAFE_NO_PAD.encode(public_key.exponent().big_endian_without_leading_zero()),
            "kid": kid,
            "alg": "RS256",
            "use": "sig",
        });
        // The RSA signer takes PKCS#1, and the key comes as PKCS#8. The PEM parser unwraps it.
        let pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            STANDARD.encode(der.as_ref())
        );
        let encoding_key = EncodingKey::from_rsa_pem(pem.as_bytes())
            .expect("Given a freshly generated PKCS#8 RSA key, parsing it should always succeed");
        Self::new(kid, Algorithm::RS256, encoding_key, &jwk)
    }

    fn new(kid: &str, algorithm: Algorithm, encoding_key: EncodingKey, jwk: &Value) -> Self {
        Self {
            kid: kid.to_owned(),
            algorithm,
            encoding_key,
            jwk: serde_json::from_value(jwk.clone())
                .expect("Given a JWK built from a generated key, parsing it should always succeed"),
        }
    }

    /// Returns the public half of the key as a JWK.
    #[must_use]
    pub fn jwk(&self) -> Jwk {
        self.jwk.clone()
    }

    /// Signs `claims` into a token whose header names the key ID.
    ///
    /// # Panics
    ///
    /// Panics if signing fails.
    #[must_use]
    pub fn sign(&self, claims: &Value) -> String {
        let mut header = Header::new(self.algorithm);
        header.kid = Some(self.kid.clone());
        self.sign_with_header(&header, claims)
    }

    /// Signs `claims` into a token whose header names no key ID.
    ///
    /// # Panics
    ///
    /// Panics if signing fails.
    #[must_use]
    pub fn sign_without_key_id(&self, claims: &Value) -> String {
        self.sign_with_header(&Header::new(self.algorithm), claims)
    }

    fn sign_with_header(&self, header: &Header, claims: &Value) -> String {
        jsonwebtoken::encode(header, claims, &self.encoding_key)
            .expect("Given a generated key and JSON claims, signing should always succeed")
    }
}
