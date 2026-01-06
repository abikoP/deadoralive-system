// テスト用ヘルパー関数
// 各テストで共通して使用するヘルパー関数を定義します

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// テスト用の一時ディレクトリを作成
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// テスト用の一時ファイルを作成
pub fn create_temp_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).expect("Failed to write temp file");
    file_path
}

/// テスト用のTelegraf設定ファイルを生成
pub fn generate_telegraf_config(urls: &[&str]) -> String {
    let mut config = String::from("# Telegraf Configuration\n\n");
    config.push_str("[[inputs.http_response]]\n");
    config.push_str("  urls = [\n");
    
    for url in urls {
        config.push_str(&format!("    \"{}\",\n", url));
    }
    
    config.push_str("  ]\n");
    config.push_str("  response_timeout = \"5s\"\n");
    config.push_str("  method = \"GET\"\n");
    
    config
}

/// テスト用のURL配列を生成
pub fn generate_test_urls(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("https://example{}.com", i))
        .collect()
}

/// テスト用の有効なURL配列を生成
pub fn generate_valid_urls() -> Vec<String> {
    vec![
        "https://example.com".to_string(),
        "http://test.org".to_string(),
        "https://api.service.com/health".to_string(),
    ]
}

/// テスト用の不正なURL配列を生成
pub fn generate_invalid_urls() -> Vec<String> {
    vec![
        "not-a-url".to_string(),
        "".to_string(),
        "   ".to_string(),
        "ftp://invalid-protocol.com".to_string(),
    ]
}

/// ファイルの内容を読み込む
pub fn read_file_content(path: &Path) -> String {
    fs::read_to_string(path).expect("Failed to read file")
}

/// ファイルが存在するか確認
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// ディレクトリが存在するか確認
pub fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_create_temp_file() {
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(temp_dir.path(), "test.txt", "test content");
        assert!(file_path.exists());
        assert_eq!(read_file_content(&file_path), "test content");
    }

    #[test]
    fn test_generate_telegraf_config() {
        let urls = vec!["https://example.com", "https://test.org"];
        let config = generate_telegraf_config(&urls);
        assert!(config.contains("https://example.com"));
        assert!(config.contains("https://test.org"));
        assert!(config.contains("[[inputs.http_response]]"));
    }

    #[test]
    fn test_generate_test_urls() {
        let urls = generate_test_urls(5);
        assert_eq!(urls.len(), 5);
        assert_eq!(urls[0], "https://example0.com");
        assert_eq!(urls[4], "https://example4.com");
    }

    #[test]
    fn test_generate_valid_urls() {
        let urls = generate_valid_urls();
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|url| url.starts_with("http")));
    }

    #[test]
    fn test_generate_invalid_urls() {
        let urls = generate_invalid_urls();
        assert!(!urls.is_empty());
    }
}
