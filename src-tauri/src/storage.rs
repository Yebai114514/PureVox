// 本地文件存储：所有持久化数据存放到 exe 同目录的 data/ 文件夹
// settings 文件中的 api_key 使用 AES-256-GCM 加密，密钥派生自机器唯一 ID
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// 设置数据（加密存储）
/// api_key 在返回前端前解密，保存到磁盘前加密
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub base_url: String,
    pub model: String,
    /// 前端视图为普通 apiKey；底层在落盘时加密
    pub api_key: String,
    pub enabled: bool,
    /// 个性化推荐总开关
    #[serde(default = "default_true")]
    pub personalization_enabled: bool,
}

fn default_true() -> bool {
    true
}

/// 加密后的数据格式：hex(nonce) + ":" + hex(ciphertext)
const ENCRYPTED_SEP: &str = ":";

/// 派生 AES-256 密钥：sha256(machine_uid + fixed_salt)
fn derive_key() -> [u8; 32] {
    let salt = "purevox-v1-salt-7f3a9e";
    let machine_id = machine_uid::get().unwrap_or_else(|_| "default-machine-id".to_string());
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(salt.as_bytes());
    hasher.finalize().into()
}

/// AES-256-GCM 加密文本，返回 hex nonce:ciphertext
fn encrypt_text(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    // 固定 nonce：每次加密生成随机 12 字节
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "{}{}{}",
        hex::encode(nonce_bytes),
        ENCRYPTED_SEP,
        hex::encode(ciphertext)
    ))
}

/// AES-256-GCM 解密 hex nonce:ciphertext
fn decrypt_text(ciphertext: &str) -> Result<String, String> {
    if ciphertext.is_empty() {
        return Ok(String::new());
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let parts: Vec<&str> = ciphertext.split(ENCRYPTED_SEP).collect();
    if parts.len() != 2 {
        return Err("invalid encrypted format".to_string());
    }
    let nonce_bytes = hex::decode(parts[0]).map_err(|e| e.to_string())?;
    let ciphertext = hex::decode(parts[1]).map_err(|e| e.to_string())?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// 获取 exe 同目录下的 data 文件夹路径
fn data_dir() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| "cannot get exe parent dir".to_string())?;
    let dir = exe_dir.join("data");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

fn file_path(name: &str) -> Result<PathBuf, String> {
    let dir = data_dir()?;
    Ok(dir.join(format!("{}.json", name)))
}

/// 读取普通 JSON 数据文件（明文）
#[tauri::command]
pub fn load_data_file(name: String) -> Result<serde_json::Value, String> {
    let path = file_path(&name)?;
    if !path.exists() {
        return Ok(serde_json::Value::Null);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

/// 写入普通 JSON 数据文件（明文）
#[tauri::command]
pub fn save_data_file(name: String, data: serde_json::Value) -> Result<(), String> {
    let path = file_path(&name)?;
    let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// 读取加密设置（返回 api_key 已解密）
#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = file_path("settings")?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut settings: AppSettings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if !settings.api_key.is_empty() {
        settings.api_key = decrypt_text(&settings.api_key)?;
    }
    Ok(settings)
}

/// 保存加密设置（api_key 会被加密）
#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = file_path("settings")?;
    let mut to_save = settings.clone();
    to_save.api_key = encrypt_text(&settings.api_key)?;
    let content = serde_json::to_string_pretty(&to_save).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}
