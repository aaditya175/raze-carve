use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct AuthConfig {
    expert_hash: String,
}

fn get_config_path() -> Result<PathBuf, String> {
    let mut path = dirs::config_dir().ok_or("Cannot find config dir")?;
    path.push("secureforge");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("expert.toml");
    Ok(path)
}

#[tauri::command]
pub async fn setup_expert_passphrase(passphrase: String) -> Result<(), String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    let config = AuthConfig {
        expert_hash: password_hash,
    };
    let toml = toml::to_string(&config).map_err(|e| e.to_string())?;

    let path = get_config_path()?;
    fs::write(&path, toml).map_err(|e| e.to_string())?;

    // Restrict config file permissions to owner-only (rw-------) so the
    // Argon2 hash and any future secrets aren't world-readable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn verify_expert_passphrase_sync(passphrase: &str) -> bool {
    let Ok(path) = get_config_path() else { return false; };
    if !path.exists() {
        return false;
    }
    let Ok(toml_str) = fs::read_to_string(path) else { return false; };
    let Ok(config) = toml::from_str::<AuthConfig>(&toml_str) else { return false; };
    let Ok(parsed_hash) = PasswordHash::new(&config.expert_hash) else { return false; };
    Argon2::default()
        .verify_password(passphrase.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn is_expert_configured_sync() -> bool {
    let Ok(path) = get_config_path() else { return false; };
    path.exists()
}

#[tauri::command]
pub async fn verify_expert_passphrase(passphrase: String) -> Result<bool, String> {
    Ok(verify_expert_passphrase_sync(&passphrase))
}

#[tauri::command]
pub async fn is_expert_configured() -> Result<bool, String> {
    Ok(is_expert_configured_sync())
}
