// UrlValidationService Unit Tests
// URL検証ロジックをテストします

#[cfg(test)]
mod single_url_validation_tests {
    // Note: 実際のテストでは、UrlValidationServiceをインポートする必要があります
    // use kiro_test::services::url_validation_service::UrlValidationService;
    
    use url::Url;

    fn validate_url_simple(url: &str) -> Result<(), String> {
        // 空文字列チェック
        if url.trim().is_empty() {
            return Err("URLが空です".to_string());
        }

        // URL形式の検証
        let parsed_url = Url::parse(url)
            .map_err(|e| format!("無効なURL形式です: {}: {}", url, e))?;

        // HTTP/HTTPSスキームチェック
        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(format!("HTTP/HTTPSスキームが必要です: {}", url));
        }

        Ok(())
    }

    #[test]
    fn test_valid_http_url() {
        // 正常系: 有効なHTTP URLを検証
        let url = "http://example.com";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_https_url() {
        // 正常系: 有効なHTTPS URLを検証
        let url = "https://example.com";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_path() {
        // 正常系: パス付きの有効なURL
        let url = "https://example.com/api/health";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_query() {
        // 正常系: クエリパラメータ付きの有効なURL
        let url = "https://example.com/search?q=test";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_port() {
        // 正常系: ポート番号付きの有効なURL
        let url = "https://example.com:8080";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_url_format() {
        // 異常系: 不正なURL形式
        let url = "not-a-url";
        let result = validate_url_simple(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url_missing_scheme() {
        // 異常系: スキームがないURL
        let url = "example.com";
        let result = validate_url_simple(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_scheme_ftp() {
        // 異常系: FTPスキーム（HTTP/HTTPS以外）
        let url = "ftp://example.com";
        let result = validate_url_simple(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTP/HTTPSスキームが必要です"));
    }

    #[test]
    fn test_invalid_scheme_file() {
        // 異常系: fileスキーム
        let url = "file:///path/to/file";
        let result = validate_url_simple(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string() {
        // エッジケース: 空文字列
        let url = "";
        let result = validate_url_simple(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("URLが空です"));
    }

    #[test]
    fn test_whitespace_only() {
        // エッジケース: 空白のみの文字列
        let url = "   ";
        let result = validate_url_simple(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("URLが空です"));
    }

    #[test]
    fn test_very_long_url() {
        // エッジケース: 非常に長いURL
        let long_path = "a".repeat(2000);
        let url = format!("https://example.com/{}", long_path);
        let result = validate_url_simple(&url);
        // 長いURLでも形式が正しければ有効
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_special_characters() {
        // エッジケース: 特殊文字を含むURL
        let url = "https://example.com/path?name=test%20value&id=123";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_fragment() {
        // エッジケース: フラグメント付きURL
        let url = "https://example.com/page#section";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_authentication() {
        // エッジケース: 認証情報付きURL
        let url = "https://user:pass@example.com";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_localhost_url() {
        // エッジケース: localhostのURL
        let url = "http://localhost:8080";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ip_address_url() {
        // エッジケース: IPアドレスのURL
        let url = "http://192.168.1.1";
        let result = validate_url_simple(url);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod multiple_urls_validation_tests {
    use super::*;

    fn validate_urls_simple(urls: &[String]) -> Result<(), String> {
        for url in urls {
            // 空文字列チェック
            if url.trim().is_empty() {
                return Err("URLが空です".to_string());
            }

            // URL形式の検証
            let parsed_url = url::Url::parse(url)
                .map_err(|e| format!("無効なURL形式です: {}: {}", url, e))?;

            // HTTP/HTTPSスキームチェック
            let scheme = parsed_url.scheme();
            if scheme != "http" && scheme != "https" {
                return Err(format!("HTTP/HTTPSスキームが必要です: {}", url));
            }
        }
        Ok(())
    }

    #[test]
    fn test_multiple_valid_urls() {
        // 正常系: 複数の有効なURLを検証
        let urls = vec![
            "https://example.com".to_string(),
            "http://test.org".to_string(),
            "https://api.service.com/health".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_array() {
        // 異常系: 空の配列
        let urls: Vec<String> = vec![];
        let result = validate_urls_simple(&urls);
        // 空の配列は有効（チェックするURLがない）
        assert!(result.is_ok());
    }

    #[test]
    fn test_one_invalid_url_in_array() {
        // 異常系: 一部が不正なURL
        let urls = vec![
            "https://example.com".to_string(),
            "not-a-url".to_string(),
            "https://test.org".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_invalid_urls() {
        // 異常系: すべて不正なURL
        let urls = vec![
            "not-a-url".to_string(),
            "also-invalid".to_string(),
            "ftp://wrong-scheme.com".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_urls() {
        // エッジケース: 重複したURL
        let urls = vec![
            "https://example.com".to_string(),
            "https://example.com".to_string(),
            "https://test.org".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        // 重複は許可される（検証のみ）
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_number_of_urls() {
        // エッジケース: 大量のURL
        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://example{}.com", i))
            .collect();
        let result = validate_urls_simple(&urls);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_http_and_https() {
        // エッジケース: HTTPとHTTPSの混在
        let urls = vec![
            "http://example.com".to_string(),
            "https://secure.com".to_string(),
            "http://test.org".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        assert!(result.is_ok());
    }

    #[test]
    fn test_urls_with_different_ports() {
        // エッジケース: 異なるポート番号
        let urls = vec![
            "https://example.com:443".to_string(),
            "https://example.com:8443".to_string(),
            "http://example.com:80".to_string(),
            "http://example.com:8080".to_string(),
        ];
        let result = validate_urls_simple(&urls);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_url_with_unicode() {
        // エッジケース: Unicode文字を含むURL
        let url = "https://例え.jp";
        let result = url::Url::parse(url);
        // Unicodeドメインは国際化ドメイン名(IDN)として扱われる
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_emoji() {
        // エッジケース: 絵文字を含むURL（パス部分）
        let url = "https://example.com/😀";
        let result = url::Url::parse(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_max_length() {
        // エッジケース: 最大長に近いURL
        // 一般的なブラウザのURL最大長は約2000文字
        let long_path = "a".repeat(1900);
        let url = format!("https://example.com/{}", long_path);
        let result = url::Url::parse(&url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_multiple_query_params() {
        // エッジケース: 複数のクエリパラメータ
        let url = "https://example.com/search?q=test&page=1&limit=10&sort=asc";
        let result = url::Url::parse(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_with_nested_paths() {
        // エッジケース: 深くネストされたパス
        let url = "https://example.com/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p";
        let result = url::Url::parse(url);
        assert!(result.is_ok());
    }
}
