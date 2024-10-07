use dkn_p2p::libp2p_identity;
use ecies::PublicKey;
use eyre::{Context, Result};
use libsecp256k1::{Message, SecretKey};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Generic SHA256 function.
#[inline(always)]
pub fn sha256hash(data: impl AsRef<[u8]>) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Generic KECCAK256 function.
#[inline(always)]
pub fn keccak256hash(data: impl AsRef<[u8]>) -> [u8; 32] {
    Keccak256::digest(data).into()
}

/// Given a secp256k1 public key, finds the corresponding Ethereum address.
///
/// Internally, the public key is serialized in uncompressed format at 65 bytes (0x04 || x || y),
/// and then (x || y) is hashed using Keccak256. The last 20 bytes of this hash is taken as the address.
#[inline]
pub fn to_address(public_key: &PublicKey) -> [u8; 20] {
    let public_key_xy = &public_key.serialize()[1..];
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&keccak256hash(public_key_xy)[12..32]);
    addr
}

/// Shorthand to sign a digest (bytes) with node's secret key and return signature & recovery id
/// serialized to 65 byte hex-string.
#[inline]
pub fn sign_bytes_recoverable(message: &[u8; 32], secret_key: &SecretKey) -> String {
    let message = Message::parse(message);
    let (signature, recid) = libsecp256k1::sign(&message, secret_key);

    format!(
        "{}{}",
        hex::encode(signature.serialize()),
        hex::encode([recid.serialize()])
    )
}

/// Shorthand to encrypt bytes with a given public key.
/// Returns hexadecimal encoded ciphertext.
#[inline]
pub fn encrypt_bytes(data: impl AsRef<[u8]>, public_key: &PublicKey) -> Result<String> {
    ecies::encrypt(&public_key.serialize(), data.as_ref())
        .wrap_err("could not encrypt data")
        .map(hex::encode)
}

/// Converts a `libsecp256k1::SecretKey` to a `libp2p_identity::secp256k1::Keypair`.
/// To do this, we serialize the secret key and create a new keypair from it.
#[inline]
pub fn secret_to_keypair(secret_key: &SecretKey) -> libp2p_identity::Keypair {
    let bytes = secret_key.serialize();

    let secret_key = dkn_p2p::libp2p_identity::secp256k1::SecretKey::try_from_bytes(bytes)
        .expect("Failed to create secret key");
    libp2p_identity::secp256k1::Keypair::from(secret_key).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecies::{decrypt, encrypt};
    use hex::decode;
    use libsecp256k1::{recover, sign, verify, Message, PublicKey, SecretKey};

    const DUMMY_SECRET_KEY: &[u8; 32] = b"driadriadriadriadriadriadriadria";
    const MESSAGE: &[u8] = b"hello world";

    #[test]
    fn test_hash() {
        // sha256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let expected = decode(expected).expect("Should decode hex string.");
        assert_eq!(sha256hash(MESSAGE), expected.as_slice());
    }

    #[test]
    fn test_address() {
        let sk = SecretKey::parse_slice(DUMMY_SECRET_KEY).expect("Should parse key.");
        let pk = PublicKey::from_secret_key(&sk);
        let addr = to_address(&pk);
        assert_eq!(
            "D79Fdf178547614CFdd0dF6397c53569716Bd596".to_lowercase(),
            hex::encode(addr)
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let sk = SecretKey::parse_slice(DUMMY_SECRET_KEY).expect("Should parse private key slice.");
        let pk = PublicKey::from_secret_key(&sk);
        let (sk, pk) = (&sk.serialize(), &pk.serialize());

        let ciphertext = encrypt(pk, MESSAGE).expect("Should encrypt.");
        let plaintext = decrypt(sk, &ciphertext).expect("Should decyrpt.");
        assert_eq!(MESSAGE, plaintext.as_slice());
    }

    #[test]
    fn test_sign_verify() {
        let secret_key =
            SecretKey::parse_slice(DUMMY_SECRET_KEY).expect("Should parse private key slice.");

        // sign the message using the secret key
        let digest = sha256hash(MESSAGE);
        let message = Message::parse_slice(&digest).expect("Should parse message.");
        let (signature, recid) = sign(&message, &secret_key);

        // recover verifying key (public key) from signature
        let expected_public_key = PublicKey::from_secret_key(&secret_key);
        let recovered_public_key =
            recover(&message, &signature, &recid).expect("Should recover public key.");
        assert_eq!(expected_public_key, recovered_public_key);

        // verify the signature
        let public_key = recovered_public_key;
        assert!(
            verify(&message, &signature, &public_key),
            "Could not verify signature."
        );
    }

    #[test]
    #[ignore = "run only with profiler if wanted"]
    fn test_memory_usage() {
        let secret_key =
            SecretKey::parse_slice(DUMMY_SECRET_KEY).expect("Should parse private key slice.");
        let public_key = PublicKey::from_secret_key(&secret_key);

        // sign the message using the secret key
        let digest = sha256hash(MESSAGE);
        let message = Message::parse_slice(&digest).expect("Should parse message.");
        let (signature, _) = sign(&message, &secret_key);

        // verify signature with context
        for _ in 0..1_000_000 {
            let ok = verify(&message, &signature, &public_key);
            assert!(ok);
        }
    }
}
