// src-tauri/src/signing.rs
use crate::crypto_types::{
    SignatureAlgorithm, SignatureFormat, SigningOptions, VerificationResult,
};
use crate::key_management::{
    decrypt_data, get_key_storage_dir, get_metadata_path, read_metadata, SALT_LEN,
}; // Import necessary helpers
use anyhow::{bail, Context, Result};
use signature::SignatureEncoding;
use std::fs;
use std::str::FromStr;
// Use Manager trait to get AppHandle features
use uuid::Uuid;

// --- Hashing ---
use sha2::{Digest, Sha256};

// --- RSA ---
use rsa::pkcs1v15::SigningKey as RsaSigningKey;
use rsa::pkcs1v15::VerifyingKey as RsaVerifyingKey;
use rsa::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _}; // Use trait import
use rsa::sha2::Sha256 as RsaSha256; // Hash used in RSA padding scheme
use rsa::{RsaPrivateKey, RsaPublicKey};

// --- ECDSA P-256 ---
use p256::ecdsa::{
    signature::Signer,
    signature::Verifier,
    Signature as EcdsaSignature, // Use alias
    SigningKey as EcdsaSigningKey,
    VerifyingKey as EcdsaVerifyingKey, // Use alias
};

// --- Ed25519 ---
// Use ed25519-dalek for direct signing/verification if you generated with it
// If you used the `ed25519` crate with `pkcs8` feature:
// use ed25519::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};
// use ed25519::{Signature as EdSignature, SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey};
// Using dalek here as it was in the modified generate code:
use ed25519_dalek::{
    Signature as EdSignature, SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey,
};

// --- PEM ---
use pem_rfc7468::{decode_vec, PemLabel}; // Import decode_vec
                                         // Use LineEnding from pkcs8::der::pem

// Constants for PEM tags (should match key_management)
const SPKI_PEM_TAG: &str = rsa::pkcs8::SubjectPublicKeyInfoRef::PEM_LABEL;

// --- Tauri Commands ---

#[tauri::command(rename_all="camelCase")]
pub fn sign_document(
    app_handle: tauri::AppHandle,
    document_path: String,
    key_id: Uuid,
    password: String,
    output_path: String,
    options: SigningOptions, // Keep options for future extensibility
) -> Result<(), String> {
    log::info!(
        "Signing document '{}' with key ID {}",
        document_path,
        key_id
    );
    if password.is_empty() {
        return Err("Password cannot be empty.".to_string());
    }

    match options.format {
        SignatureFormat::Detached => {
            sign_document_detached(
                &app_handle, // Pass handle for path resolution
                &document_path,
                key_id,
                &password,
                &output_path,
            )
            .map_err(|e| {
                log::error!("Failed to sign document: {:?}", e);
                e.to_string()
            })
        } // SignatureFormat::Embedded => Err("Embedded signatures are not supported yet.".to_string()),
    }
}

fn sign_document_detached(
    app_handle: &tauri::AppHandle,
    document_path_str: &str,
    key_id: Uuid,
    password: &str,
    output_path_str: &str,
) -> Result<()> {
    // 1. Find key metadata and parse algorithm
    let metadata_path = get_metadata_path(app_handle)?;
    let metadata = read_metadata(&metadata_path)?
        .into_iter()
        .find(|m| m.key_id == key_id)
        .ok_or_else(|| anyhow::anyhow!("Key with ID {} not found", key_id))?;

    let algorithm = SignatureAlgorithm::from_str(&metadata.algorithm).with_context(|| {
        format!(
            "Invalid algorithm '{}' found in metadata for key {}",
            metadata.algorithm, key_id
        )
    })?;

    // 2. Read and decrypt private key DER bytes
    let key_storage_dir = get_key_storage_dir(app_handle)?;
    let private_key_path = key_storage_dir.join(&metadata.encrypted_private_key_path);
    let mut encrypted_private_key_bytes = fs::read(&private_key_path) // Read into mutable Vec
        .with_context(|| {
            format!(
                "Failed to read encrypted private key file: {:?}",
                private_key_path
            )
        })?;
    let mut salt = [0; SALT_LEN];
    hex::decode_to_slice(&metadata.salt_hex, &mut salt)
        .context("Failed to decode salt from hex")?;

    // Decrypt in place - passing mutable Vec
    decrypt_data(&mut encrypted_private_key_bytes, password, &salt)
        .context("Failed to decrypt private key (check password)")?;
    // encrypted_private_key_bytes now holds the decrypted DER bytes

    // 3. Read document data (needed for hashing or direct signing)
    let document_bytes = fs::read(document_path_str)
        .with_context(|| format!("Failed to read document file: {}", document_path_str))?;

    // 4. Algorithm-specific signing

    log::debug!("Performing signing with algorithm: {}", algorithm);

    let signature_bytes = match algorithm {
        SignatureAlgorithm::RsaPkcs1Sha256 => {
            // Parse private key
            let private_key = RsaPrivateKey::from_pkcs8_der(&encrypted_private_key_bytes)
                .context("Failed to parse decrypted data as RSA private key")?;
            // Create signing key with specific padding/hash
            let signing_key = RsaSigningKey::<RsaSha256>::new(private_key);
            // Hash the document
            let mut hasher = Sha256::new();
            hasher.update(&document_bytes);
            let digest = hasher.finalize();
            // Sign the hash
            signing_key.sign(digest.as_slice()).to_vec()
        }
        SignatureAlgorithm::EcdsaP256Sha256 => {
            // Parse private key
            let private_key = EcdsaSigningKey::from_pkcs8_der(&encrypted_private_key_bytes)
                .context("Failed to parse decrypted data as ECDSA P-256 private key")?;
            // Hash the document
            let mut hasher = Sha256::new();
            hasher.update(&document_bytes);
            let digest = hasher.finalize();
            // Sign the hash - P256 SigningKey implements Signer trait
            let signature: EcdsaSignature = private_key.sign(digest.as_slice());
            signature.to_vec()
        }
        SignatureAlgorithm::Ed25519 => {
            // Parse private key
            let private_key = EdSigningKey::from_pkcs8_der(&encrypted_private_key_bytes)
                .context("Failed to parse decrypted data as Ed25519 private key")?;
            // Sign the message directly (no pre-hashing) using dalek's Signer trait
            let signature = private_key.sign(&document_bytes);
            signature.to_bytes().to_vec()
        }
    };

    // 5. Write signature to output file
    fs::write(output_path_str, &signature_bytes)
        .with_context(|| format!("Failed to write signature file: {}", output_path_str))?;

    log::info!(
        "Document successfully signed with {}. Signature saved to {}",
        algorithm,
        output_path_str
    );
    Ok(())
}

#[tauri::command(rename_all="camelCase")]
pub fn verify_signature(
    app_handle: tauri::AppHandle,
    document_path: String,
    signature_path: String,
    key_id: Uuid, // Key ID whose public key should be used
) -> Result<VerificationResult, String> {
    // Return Result<Ok, ErrString> to Tauri
    log::info!(
        "Verifying signature for document '{}' using key ID {}",
        document_path,
        key_id
    );

    // Use map_or_else for cleaner error handling back to Tauri
    verify_signature_detached(&app_handle, &document_path, &signature_path, key_id).map_err(
        // If internal function returns Err (process error)
        |e| {
            log::error!("Verification process failed upstream: {:?}", e);
            e.to_string()
        },
    )
}

// Internal verification function returns Result<VerificationResult>
fn verify_signature_detached(
    app_handle: &tauri::AppHandle,
    document_path_str: &str,
    signature_path_str: &str,
    key_id: Uuid,
) -> Result<VerificationResult> {
    // Return internal Result
    // 1. Find key metadata and parse algorithm
    let metadata_path = get_metadata_path(app_handle)?;
    let metadata = read_metadata(&metadata_path)?
        .into_iter()
        .find(|m| m.key_id == key_id)
        .ok_or_else(|| anyhow::anyhow!("Key with ID {} not found", key_id))?;

    let algorithm = SignatureAlgorithm::from_str(&metadata.algorithm).with_context(|| {
        format!(
            "Invalid algorithm '{}' found in metadata for key {}",
            metadata.algorithm, key_id
        )
    })?;

    // 2. Read public key PEM file
    let key_storage_dir = get_key_storage_dir(app_handle)?;
    let public_key_path = key_storage_dir.join(&metadata.public_key_pem_path);
    let public_key_pem = fs::read_to_string(&public_key_path)
        .with_context(|| format!("Failed to read public key file: {:?}", public_key_path))?;

    // 3. Decode PEM to get SPKI DER bytes
    let (label, public_key_der) = decode_vec(public_key_pem.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to decode public key PEM: {}", e))?;
    if label != SPKI_PEM_TAG {
        bail!(
            "Invalid PEM label for public key: expected '{}', found '{}'",
            SPKI_PEM_TAG,
            label
        );
    }

    // 4. Read document data
    let document_bytes = fs::read(document_path_str)
        .with_context(|| format!("Failed to read document file: {}", document_path_str))?;

    // 5. Read signature file
    let signature_bytes = fs::read(signature_path_str)
        .with_context(|| format!("Failed to read signature file: {}", signature_path_str))?;
    let signature = signature_bytes
        .as_slice()
        .try_into()
        .context("Failed to convert signature bytes")?;

    // 6. Algorithm-specific verification
    log::debug!("Performing verification with algorithm: {}", algorithm);

    let verification_result = match algorithm {
        SignatureAlgorithm::RsaPkcs1Sha256 => {
            let public_key = RsaPublicKey::from_public_key_der(&public_key_der)
                .context("Failed to parse SPKI DER as RSA public key")?;
            let verifying_key = RsaVerifyingKey::<RsaSha256>::new(public_key);
            let mut hasher = Sha256::new();
            hasher.update(&document_bytes);
            let digest = hasher.finalize();
            // Verify the hash against the signature
            verifying_key.verify(digest.as_slice(), &signature)
        }
        SignatureAlgorithm::EcdsaP256Sha256 => {
            let public_key = EcdsaVerifyingKey::from_public_key_der(&public_key_der)
                .context("Failed to parse SPKI DER as ECDSA P-256 public key")?;
            let mut hasher = Sha256::new();
            hasher.update(&document_bytes);
            let digest = hasher.finalize();
            // Try to parse the signature bytes into an ECDSA signature structure
            let signature = EcdsaSignature::from_slice(&signature_bytes)
                .context("Failed to parse signature bytes as ECDSA signature")?;
            // Verify the hash against the signature
            public_key.verify(digest.as_slice(), &signature)
        }
        SignatureAlgorithm::Ed25519 => {
            let public_key = EdVerifyingKey::from_public_key_der(&public_key_der)
                .context("Failed to parse SPKI DER as Ed25519 public key")?;
            // Try to parse the signature bytes into an Ed25519 signature structure
            let signature = EdSignature::from_slice(&signature_bytes)
                .context("Failed to parse signature bytes as Ed25519 signature")?;
            // Verify the original message against the signature
            public_key.verify(&document_bytes, &signature)
        }
    };

    // 7. Convert verification result (Ok or signature::Error) to VerificationResult struct
    match verification_result {
        Ok(_) => {
            log::info!(
                "Verification successful for document: {}",
                document_path_str
            );
            Ok(VerificationResult {
                is_valid: true,
                error_message: None,
            })
        }
        Err(sig_err) => {
            // This specifically means the cryptographic verification failed
            log::warn!(
                "Verification failed for document {}: {}",
                document_path_str,
                sig_err
            );
            Ok(VerificationResult {
                is_valid: false,
                error_message: Some(format!("Signature is invalid: {}", sig_err)),
            })
        }
    }
}
