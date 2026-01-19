# テスト実行ガイド

このドキュメントでは、deadoralive-systemのテスト実行方法について説明します。

## 目次

1. [テスト概要](#テスト概要)
2. [テスト環境のセットアップ](#テスト環境のセットアップ)
3. [テストの実行方法](#テストの実行方法)
4. [テストカバレッジ](#テストカバレッジ)
5. [CI/CDでのテスト実行](#cicdでのテスト実行)

## テスト概要

本プロジェクトでは、以下の4種類のテストを実装しています：

### 1. Unit Tests（単体テスト）
個々のコンポーネントの動作を検証します。

- **ConfigService**: 設定ファイルの読み書き、バックアップ、ロールバック
- **UrlValidationService**: URL検証ロジック
- **Telegraf Controller**: Docker操作、エラーハンドリング

### 2. Property-Based Tests（プロパティベーステスト）
ランダムな入力で正確性を検証します（最小100回実行）。

- **Property 1: Atomicity**: 設定ファイル更新の原子性
- **Property 2: SIGHUP Delivery**: SIGHUPシグナルの確実な送信
- **Property 5: Rollback**: エラー時の設定ロールバック

### 3. Integration Tests（統合テスト）
複数のコンポーネント間の連携を検証します。

- **Manager ↔ Telegraf**: 設定更新とリロードの統合
- **Manager ↔ Docker Socket**: Docker操作の統合

### 4. End-to-End Tests（E2Eテスト）
システム全体の動作を検証します。

- 全サービスの起動確認
- 完全なワークフローの検証
- エラーシナリオの検証

## テスト環境のセットアップ

### 前提条件

- Rust 1.70以上
- Docker（統合テスト・E2Eテスト用）
- Docker Compose（E2Eテスト用）

### 依存関係のインストール

```bash
cd manager
cargo build --tests
```

## テストの実行方法

### すべてのテストを実行

```bash
cargo test
```

### Unit Testsのみ実行

```bash
cargo test --lib unit
```

### Property-Based Testsのみ実行

```bash
cargo test --lib property
```

**注意**: Property-Based Testsは最小100回実行されるため、実行時間が長くなります。

### Integration Testsのみ実行

```bash
cargo test --lib integration
```

**注意**: Integration Testsの一部はDocker環境が必要です。`#[ignore]`属性が付いているテストは、以下のコマンドで実行できます：

```bash
cargo test --lib integration -- --ignored
```

### E2E Testsのみ実行

```bash
# 前提: Docker Composeで全サービスを起動
docker compose up -d

# E2Eテストを実行
cargo test --test e2e_tests -- --ignored
```

### 特定のテストを実行

```bash
# テスト名で絞り込み
cargo test test_config_file_read

# モジュール名で絞り込み
cargo test config_service_tests
```

### 詳細な出力で実行

```bash
cargo test -- --nocapture
```

### 並列実行を無効化

```bash
cargo test -- --test-threads=1
```

## テストカバレッジ

### カバレッジレポートの生成

#### 方法1: cargo-tarpaulin（推奨）

```bash
# インストール
cargo install cargo-tarpaulin

# カバレッジレポート生成
cargo tarpaulin --out Html --output-dir coverage
```

生成されたレポートは `coverage/index.html` で確認できます。

#### 方法2: cargo-llvm-cov

```bash
# インストール
cargo install cargo-llvm-cov

# カバレッジレポート生成
cargo llvm-cov --html --output-dir coverage
```

### カバレッジ目標

- **全体**: 80%以上
- **Unit Tests**: 90%以上
- **Integration Tests**: 70%以上

## CI/CDでのテスト実行

GitHub Actionsでは、以下のテストが自動実行されます：

### プルリクエスト時

1. **Format Check**: `cargo fmt --check`
2. **Lint Check**: `cargo clippy -- -D warnings`
3. **Unit Tests**: `cargo test --lib unit`
4. **Property-Based Tests**: `cargo test --lib property`
5. **Integration Tests**: `cargo test --lib integration`（Docker不要なもののみ）

### マージ後（main/developブランチ）

1. 上記すべて
2. **E2E Tests**: `cargo test --test e2e_tests -- --ignored`（Docker Compose環境で実行）

## テストのベストプラクティス

### 1. テストの独立性

各テストは独立して実行可能であり、他のテストに依存しないようにしてください。

```rust
#[test]
fn test_example() {
    // テスト用の一時ファイルを作成
    let temp_dir = TempDir::new().unwrap();
    
    // テスト実行
    // ...
    
    // TempDirは自動的にクリーンアップされる
}
```

### 2. テストの命名規則

- テスト関数名: `test_<機能>_<シナリオ>`
- 例: `test_config_file_read_success`, `test_url_validation_invalid_format`

### 3. テストの構造

```rust
#[test]
fn test_example() {
    // Arrange: テストデータの準備
    let input = "test data";
    
    // Act: テスト対象の実行
    let result = function_under_test(input);
    
    // Assert: 結果の検証
    assert_eq!(result, expected_value);
}
```

### 4. エラーメッセージ

```rust
assert!(
    condition,
    "Descriptive error message: expected {}, got {}",
    expected,
    actual
);
```

## トラブルシューティング

### テストが失敗する場合

1. **依存関係の確認**
   ```bash
   cargo clean
   cargo build --tests
   ```

2. **Docker環境の確認**（Integration/E2Eテスト）
   ```bash
   docker version
   docker compose version
   docker compose ps
   ```

3. **詳細なログ出力**
   ```bash
   RUST_LOG=debug cargo test -- --nocapture
   ```

### よくある問題

#### 問題: "Docker is not available"

**解決策**: Dockerが起動していることを確認してください。

```bash
docker version
```

#### 問題: "Telegraf container is not running"

**解決策**: Docker Composeで全サービスを起動してください。

```bash
docker compose up -d
docker compose ps
```

#### 問題: Property-Based Testsが遅い

**解決策**: これは正常な動作です。Property-Based Testsは最小100回実行されるため、時間がかかります。

## 参考資料

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest Documentation](https://docs.rs/proptest/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

## お問い合わせ

テストに関する質問や問題がある場合は、プロジェクトのIssueトラッカーで報告してください。
