const SERVICE: &str = "com.andydev404.cleft";
const ACCOUNT: &str = "clips-db-key";

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// Random passphrase for SQLCipher's PRAGMA key. Not stored in plaintext
// anywhere on disk — only ever in the Keychain, fetched at startup.
fn generate_key() -> String {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    hex_encode(&bytes)
}

#[cfg(target_os = "macos")]
pub fn get_or_create_db_key() -> String {
    use security_framework::passwords::{get_generic_password, set_generic_password};

    if let Ok(bytes) = get_generic_password(SERVICE, ACCOUNT) {
        return String::from_utf8(bytes).expect("stored key should be valid utf8");
    }

    let key = generate_key();
    set_generic_password(SERVICE, ACCOUNT, key.as_bytes())
        .expect("failed to store db encryption key in Keychain");
    key
}

#[cfg(not(target_os = "macos"))]
pub fn get_or_create_db_key() -> String {
    unimplemented!("Keychain-backed encryption is macOS-only in V1")
}
