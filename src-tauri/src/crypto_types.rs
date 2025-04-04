use anyhow::bail;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr}; // Import necessary traits
use uuid::Uuid;

// --- Public Data Transfer Objects (DTOs) ---
// These are the types exchanged between Rust backend and Svelte frontend via Tauri commands.

/// Represents detailed information about a cryptographic key (excluding the private key).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON if preferred by frontend
pub struct KeyDetails {
    pub info: KeyInfo,
    /// The public key encoded in PEM (SPKI) format.
    pub public_key_pem: String,
}

/// Represents information returned immediately after successfully generating or importing a key pair.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON if preferred by frontend
pub struct KeyInfo {
    /// Unique identifier for the key pair.
    pub key_id: Uuid,
    /// User-defined name for the key.
    pub name: String,
    /// The signature algorithm associated with this key.
    /// Matches the `Display` representation of `SignatureAlgorithm`.
    pub algorithm: String,
    /// Timestamp (UTC) when the key was generated or imported.
    pub created_at: DateTime<Utc>,
}

/// Enumerates the supported signature algorithms within the application.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON if preferred by frontend
pub enum SignatureAlgorithm {
    /// RSA signature scheme with PKCS#1 v1.5 padding and SHA-256 hashing.
    /// Typically used with 2048-bit keys or larger.
    RsaPkcs1Sha256,
    /// Elliptic Curve Digital Signature Algorithm (ECDSA) using the NIST P-256 curve
    /// and SHA-256 hashing.
    EcdsaP256Sha256,
    /// Edwards-curve Digital Signature Algorithm (EdDSA) using the Ed25519 curve.
    /// Hashing is implicitly defined by the Ed25519 scheme.
    Ed25519,
}

impl fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignatureAlgorithm::RsaPkcs1Sha256 => write!(f, "RSA-PKCS1-SHA256"),
            SignatureAlgorithm::EcdsaP256Sha256 => write!(f, "ECDSA-P256-SHA256"),
            SignatureAlgorithm::Ed25519 => write!(f, "Ed25519"),
        }
    }
}

impl FromStr for SignatureAlgorithm {
    type Err = anyhow::Error;

    /// Parses a string representation into a `SignatureAlgorithm`.
    /// Case-insensitive and ignores hyphens/underscores for flexibility.
    /// Recognizes common aliases (e.g., "RSA", "P256").
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Normalize the input string: uppercase, remove separators
        let normalized = s.to_uppercase().replace(['-', '_'], "");
        match normalized.as_str() {
            // RSA Aliases
            "RSAPKCS1SHA256" | "RSA2048" | "RSA" => Ok(SignatureAlgorithm::RsaPkcs1Sha256),
            // ECDSA Aliases
            "ECDSAP256SHA256" | "P256" | "ECDSAP256" | "ECP256" => {
                Ok(SignatureAlgorithm::EcdsaP256Sha256)
            }
            // Ed25519 Aliases
            "ED25519" => Ok(SignatureAlgorithm::Ed25519),
            // Unrecognized
            _ => bail!("Unsupported or unrecognized signature algorithm: {}", s),
        }
    }
}

/// Specifies the desired format for the output signature.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON if preferred by frontend
pub enum SignatureFormat {
    /// Signature is stored in a separate file (e.g., `.sig`).
    Detached,
    // /// Signature is embedded within the document (e.g., PDF PAdES).
    // /// Not currently implemented.
    // Embedded,
}

/// Options provided when invoking the signing command.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON
pub struct SigningOptions {
    /// The desired output format for the signature.
    pub format: SignatureFormat,
    // --- Future Extensions ---
    // pub use_timestamp: bool,
    // pub tsa_url: Option<String>,
    // pub signature_level: Option<SignatureLevel>, // e.g., for PAdES B-B, B-T, B-LT, B-LTA
}

/// Represents the outcome of a signature verification attempt.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Optional: use camelCase for JSON
pub struct VerificationResult {
    /// Indicates whether the cryptographic verification was successful.
    /// `true` if the signature matches the document and public key, `false` otherwise.
    pub is_valid: bool,
    /// Provides details if verification failed or if a processing error occurred.
    /// `None` if verification was successful. Contains either a signature validation error
    /// message or a processing error message.
    #[serde(skip_serializing_if = "Option::is_none")] // Don't include in JSON if None
    pub error_message: Option<String>,
    // --- Future Extensions ---
    // /// Information extracted from the signer's certificate (if available).
    // pub signer_info: Option<SignerDetails>,
    // /// Information extracted from a timestamp token (if present and valid).
    // pub timestamp_info: Option<TimestampDetails>,
}

// --- Internal Metadata Struct ---
// This struct is used internally by the backend to manage key storage details.
// It is NOT directly exposed to the frontend via Tauri commands. Marked `pub(crate)`.

/// Internal structure for storing metadata about a key pair within the application's storage.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct KeyMetadata {
    /// Unique identifier for the key pair.
    pub(crate) key_id: Uuid,
    /// User-defined name for the key.
    pub(crate) name: String,
    /// Path (relative to the key storage directory) to the PEM-encoded public key file.
    pub(crate) public_key_pem_path: String,
    /// Path (relative to the key storage directory) to the file containing the encrypted private key.
    pub(crate) encrypted_private_key_path: String,
    /// The signature algorithm associated with this key (matches `SignatureAlgorithm::to_string()`).
    pub(crate) algorithm: String,
    /// Timestamp (UTC) when the key was generated or imported.
    pub(crate) created_at: DateTime<Utc>,
    /// The salt used for deriving the encryption key from the password, hex-encoded.
    pub(crate) salt_hex: String,
}
