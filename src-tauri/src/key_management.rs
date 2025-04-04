use crate::crypto_types::{KeyDetails, KeyInfo, KeyMetadata, SignatureAlgorithm};
use aead::{AeadMutInPlace, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use pbkdf2::pbkdf2_hmac;
use pkcs8::der::EncodePem;
use pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding, SecretDocument};
use rsa::rand_core::RngCore;
use rsa::RsaPrivateKey;
use sha2::Sha256;
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tauri::Manager;
use uuid::Uuid;

const KEY_METADATA_FILENAME: &str = "key_metadata.json";
const KEY_STORAGE_DIR: &str = "keys"; // 密钥存储目录

// const KEYRING_SERVICE_NAME: &str = "my-digital-signature-app";
const PBKDF2_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };
pub const SALT_LEN: usize = 16;
const AES_KEY_LEN: usize = 32; // Explicit AES-256 key length
const NONCE_LEN: usize = 12; // AES-GCM standard nonce length is 12 bytes (96 bits)

#[tauri::command(rename_all = "camelCase")]
pub fn generate_key_pair(
    app_handle: tauri::AppHandle,
    name: String,
    alg_str: String,
    password: String,
) -> Result<KeyDetails, String> {
    log::info!(
        "Generating key pair with name: {}, algorithm: {}",
        name,
        alg_str
    );
    if password.is_empty() {
        return Err("Password cannot be empty.".to_string());
    }

    // Parse the algorithm string
    let algorithm = match SignatureAlgorithm::from_str(&alg_str) {
        Ok(alg) => alg,
        Err(e) => {
            log::error!("Invalid algorithm requested: {}", e);
            return Err(e.to_string());
        }
    };

    _generate_key_pair(&app_handle, name, algorithm, password).map_err(|e| {
        log::error!("Failed to generate key pair: {:?}", e);
        e.to_string()
    })
}

fn _generate_key_pair(
    app_handle: &tauri::AppHandle,
    name: String,
    algorithm: SignatureAlgorithm,
    password: String,
) -> Result<KeyDetails> {
    let mut rng = OsRng;

    // --- Algorithm-Specific Key Generation ---
    // These variables will be populated by the match block
    let private_key_pkcs8_der: SecretDocument; // Use Opaque struct for DER bytes
    let public_key_spki_der: pkcs8::SubjectPublicKeyInfoRef<'_>; // Use borrowed DER ref initially
    let generated_public_key_der_bytes: Vec<u8>; // Store owned public key DER for PEM encoding

    match algorithm {
        SignatureAlgorithm::RsaPkcs1Sha256 => {
            log::debug!("Generating RSA-2048 key pair");
            let bits = 2048;
            let private_key =
                RsaPrivateKey::new(&mut rng, bits).context("Failed to generate RSA private key")?;
            let public_key = private_key.to_public_key();

            private_key_pkcs8_der = private_key
                .to_pkcs8_der()
                .context("Failed to encode RSA private key to PKCS#8 DER")?;
            // EncodePublicKey returns Result<Document>, need owned bytes
            let pub_der_doc = public_key
                .to_public_key_der()
                .context("Failed to encode RSA public key to SPKI DER")?;
            generated_public_key_der_bytes = pub_der_doc.into_vec(); // Convert Document to Vec<u8>
            public_key_spki_der = pkcs8::SubjectPublicKeyInfoRef::try_from(
                generated_public_key_der_bytes.as_slice(),
            )?;
        }
        SignatureAlgorithm::EcdsaP256Sha256 => {
            log::debug!("Generating ECDSA P-256 key pair");
            let private_key = p256::ecdsa::SigningKey::random(&mut rng); // Generate ECDSA P-256 key
            let public_key = private_key.verifying_key(); // Get the verifying/public key

            private_key_pkcs8_der = private_key
                .to_pkcs8_der()
                .context("Failed to encode ECDSA private key to PKCS#8 DER")?;
            let pub_der_doc = public_key
                .to_public_key_der()
                .context("Failed to encode ECDSA public key to SPKI DER")?;
            generated_public_key_der_bytes = pub_der_doc.into_vec();
            public_key_spki_der = pkcs8::SubjectPublicKeyInfoRef::try_from(
                generated_public_key_der_bytes.as_slice(),
            )?;
        }
        SignatureAlgorithm::Ed25519 => {
            log::debug!("Generating Ed25519 key pair");
            let private_key = ed25519_dalek::SigningKey::generate(&mut rng); // Generate Ed25519 key
            let public_key = private_key.verifying_key(); // Get the verifying/public key

            private_key_pkcs8_der = private_key
                .to_pkcs8_der()
                .context("Failed to encode Ed25519 private key to PKCS#8 DER")?;
            let pub_der_doc = public_key
                .to_public_key_der()
                .context("Failed to encode Ed25519 public key to SPKI DER")?;
            generated_public_key_der_bytes = pub_der_doc.into_vec();
            public_key_spki_der = pkcs8::SubjectPublicKeyInfoRef::try_from(
                generated_public_key_der_bytes.as_slice(),
            )?;
        }
    }

    // --- Common Logic (Post Key Generation) ---

    // 1. Encode public key DER to PEM String
    let public_key_pem_string = public_key_spki_der
        .to_pem(LineEnding::LF)
        .context("Failed to encode public key to PEM")?;

    // 2. Generate salt and encrypt the PKCS#8 DER bytes of the private key
    let mut salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut salt);
    let mut encrypted_private_key = private_key_pkcs8_der.to_bytes();
    encrypt_data(&mut encrypted_private_key, &password, &salt)?;

    // 3. Prepare storage paths (remains the same)
    let key_storage_dir = get_key_storage_dir(app_handle)?;
    let key_id = Uuid::new_v4();
    let public_key_filename = format!("{}.pub.pem", key_id);
    let private_key_filename = format!("{}.key.enc", key_id);
    let public_key_path = key_storage_dir.join(&public_key_filename);
    let private_key_path = key_storage_dir.join(&private_key_filename);

    // 4. Save public key PEM and encrypted private key (remains the same)
    fs::write(&public_key_path, &public_key_pem_string)
        .with_context(|| format!("Failed to write public key to {:?}", public_key_path))?;
    fs::write(&private_key_path, &encrypted_private_key).with_context(|| {
        format!(
            "Failed to write encrypted private key to {:?}",
            private_key_path
        )
    })?;

    // 5. Create and save metadata
    let algorithm_display_name = algorithm.to_string(); // Get display string from enum
    let metadata_entry = KeyMetadata {
        key_id,
        name: name.clone(),
        public_key_pem_path: public_key_filename,
        encrypted_private_key_path: private_key_filename,
        algorithm: algorithm_display_name, // Store the correct algorithm name
        created_at: Utc::now(),
        salt_hex: hex::encode(salt),
    };
    // ... (write metadata logic remains the same) ...
    let metadata_path = get_metadata_path(app_handle)?;
    let mut all_metadata = read_metadata(&metadata_path)?;
    all_metadata.push(metadata_entry.clone());
    write_metadata(&metadata_path, &all_metadata)?;

    log::info!(
        "Successfully generated and saved {} key pair with ID: {}",
        algorithm,
        key_id
    );

    // 6. Return KeyInfo (remains the same)
    Ok(KeyDetails {
        info: KeyInfo {
            key_id,
            name,
            algorithm: metadata_entry.algorithm,
            created_at: metadata_entry.created_at,
        },
        public_key_pem: public_key_pem_string,
    })
}

#[tauri::command(rename_all="camelCase")]
pub fn list_keys(app_handle: tauri::AppHandle) -> Result<Vec<KeyInfo>, String> {
    log::info!("Listing available keys");
    let metadata_path = get_metadata_path(&app_handle).map_err(|e| e.to_string())?;
    _list_keys(&metadata_path).map_err(|e| {
        log::error!("Failed to list keys: {:?}", e);
        e.to_string()
    })
}

fn _list_keys(metadata_path: &Path) -> Result<Vec<KeyInfo>> {
    read_metadata(metadata_path)?
        .into_iter()
        .map(|meta| {
            Ok(KeyInfo {
                key_id: meta.key_id,
                name: meta.name,
                algorithm: meta.algorithm,
                created_at: meta.created_at,
            })
        })
        .collect()
}

#[tauri::command(rename_all="camelCase")]
pub fn get_key_details(app_handle: tauri::AppHandle, key_id: Uuid) -> Result<KeyDetails, String> {
    log::info!("Getting details for key ID: {}", key_id);
    _get_key_details(&app_handle, key_id).map_err(|e| {
        log::error!("Failed to get key details for {}: {:?}", key_id, e);
        e.to_string()
    })
}

fn _get_key_details(app_handle: &tauri::AppHandle, key_id: Uuid) -> Result<KeyDetails> {
    let metadata_path = get_metadata_path(app_handle)?;
    let metadata = read_metadata(&metadata_path)?
        .into_iter()
        .find(|m| m.key_id == key_id)
        .ok_or_else(|| anyhow::anyhow!("Key with ID {} not found", key_id))?;

    let key_storage_dir = get_key_storage_dir(app_handle)?;
    let public_key_path = key_storage_dir.join(&metadata.public_key_pem_path);

    let public_key_pem = fs::read_to_string(&public_key_path)
        .with_context(|| format!("Failed to read public key file: {:?}", public_key_path))?;

    Ok(KeyDetails {
        info: KeyInfo {
            key_id: metadata.key_id,
            name: metadata.name,
            algorithm: metadata.algorithm,
            created_at: metadata.created_at,
        },
        public_key_pem,
    })
}

// --- Helper Functions ---

// 获取存储密钥元数据的文件路径
pub fn get_metadata_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).context("Failed to create app data directory")?;
    }
    Ok(data_dir.join(KEY_METADATA_FILENAME))
}

// 读取密钥元数据列表
pub fn read_metadata(path: &Path) -> Result<Vec<KeyMetadata>> {
    if !path.exists() {
        return Ok(Vec::new()); // 文件不存在，返回空列表
    }
    let content = fs::read_to_string(path).context("Failed to read metadata file")?;
    serde_json::from_str(&content).context("Failed to parse metadata JSON")
}

// 写入密钥元数据列表
pub fn write_metadata(path: &Path, metadata: &[KeyMetadata]) -> Result<()> {
    let content =
        serde_json::to_string_pretty(metadata).context("Failed to serialize metadata to JSON")?;
    fs::write(path, content).context("Failed to write metadata file")
}

// 获取存储密钥文件的目录
pub fn get_key_storage_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let path = app_handle
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;
    let keys_dir = path.join(KEY_STORAGE_DIR);
    if !keys_dir.exists() {
        fs::create_dir_all(&keys_dir).context("Failed to create keys directory")?;
    }
    Ok(keys_dir)
}

// 使用 PBKDF2 从密码和盐值派生加密密钥
fn derive_encryption_key(password: &str, salt: &[u8]) -> [u8; AES_KEY_LEN] {
    let mut key = [0u8; AES_KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS.get(), &mut key);
    key
}

// 加密数据
pub fn encrypt_data(data: &mut Vec<u8>, password: &str, salt: &[u8]) -> Result<()> {
    let key_bytes = derive_encryption_key(password, salt);
    let mut cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create AES cipher: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes); // Use OsRng
    let nonce = Nonce::from_slice(&nonce_bytes); // Create Nonce object

    // AeadInPlace encrypts the data directly
    cipher
        .encrypt_in_place(nonce, b"", data) // Use empty AAD (b"")
        .map_err(|e| anyhow::anyhow!("Failed to encrypt data: {}", e))?;

    data.extend_from_slice(&nonce_bytes); // Append nonce to the end of the data

    Ok(())
}

// 解密数据
pub fn decrypt_data(
    encrypted_data_with_nonce: &mut Vec<u8>,
    password: &str,
    salt: &[u8],
) -> Result<()> {
    if encrypted_data_with_nonce.len() < NONCE_LEN {
        bail!("Encrypted data is too short (missing nonce)");
    }
    let len = encrypted_data_with_nonce.len();
    let nonce_bytes: [u8; NONCE_LEN] = encrypted_data_with_nonce[len - NONCE_LEN..].try_into()?;
    encrypted_data_with_nonce.truncate(len - NONCE_LEN); // Remove nonce from the data
    let nonce = Nonce::from_slice(&nonce_bytes);

    let key_bytes = derive_encryption_key(password, salt);
    let mut cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create AES cipher: {}", e))?;

    cipher
        .decrypt_in_place(nonce, b"", encrypted_data_with_nonce)
        .map_err(|e| anyhow::anyhow!("Failed to decrypt data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn encrypt_decrypt_data() {
        use super::*;
        use rand::Rng;

        let password = "test_password";
        let salt: [u8; SALT_LEN] = rand::rng().random();
        let mut data = b"Hello, World!".to_vec();

        encrypt_data(&mut data, password, &salt).unwrap();

        decrypt_data(&mut data, password, &salt).unwrap();
        assert_eq!(&data, b"Hello, World!");
    }
}
