use serde::Deserialize;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Base64 反序列化函数（支持 null 值）
pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::Engine;
    // 先尝试反序列化为 Option<String>，以支持 null 值
    let opt_s: Option<String> = Deserialize::deserialize(deserializer)?;
    let s = match opt_s {
        Some(s) => s,
        None => return Ok(Vec::new()), // null 或缺失时返回空 Vec
    };
    if s.is_empty() {
        return Ok(Vec::new());
    }
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(serde::de::Error::custom)
}

/// 解压 gzip 数据
pub fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// 压缩数据为 gzip 格式
pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// 生成消息 ID（参考 Go SDK 的 GetMsgID）
pub fn generate_msg_id(user_id: &str) -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}", user_id, nanos)
}
