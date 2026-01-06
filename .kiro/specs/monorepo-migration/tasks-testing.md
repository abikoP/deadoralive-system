# Testing Implementation Plan

## Overview

このドキュメントは、deadoralive-systemのテスト実装計画です。Unit Tests、Property-Based Tests、Integration Tests、End-to-End Testsを段階的に実装します。

## Tasks

- [x] 1. テスト環境のセットアップ
  - [x] 1.1 Rustテストフレームワークの設定確認
    - `Cargo.toml`に必要な依存関係を追加
    - `proptest`クレートを追加（Property-Based Testing用）
    - テスト用のモックライブラリを追加（必要に応じて）
    - _Requirements: Testing Strategy_

  - [x] 1.2 テストディレクトリ構造の作成
    - `manager/tests/`ディレクトリを作成
    - `manager/tests/unit/`ディレクトリを作成
    - `manager/tests/integration/`ディレクトリを作成
    - `manager/tests/property/`ディレクトリを作成
    - _Requirements: Testing Strategy_

  - [x] 1.3 テスト用のヘルパー関数を作成
    - テスト用の一時ファイル作成ヘルパー
    - テスト用の設定ファイル生成ヘルパー
    - テスト用のURL生成ヘルパー
    - _Requirements: Testing Strategy_

- [x] 2. Unit Tests: ConfigService
  - [x] 2.1 設定ファイル読み込みテスト
    - 正常系: 有効な設定ファイルを読み込む
    - 異常系: ファイルが存在しない場合
    - 異常系: 読み取り権限がない場合
    - エッジケース: 空の設定ファイル
    - _Requirements: 11.1, Testing Strategy_

  - [x] 2.2 設定ファイル書き込みテスト
    - 正常系: 設定ファイルに書き込む
    - 異常系: 書き込み権限がない場合
    - 異常系: ディスク容量不足の場合
    - _Requirements: 11.1, Testing Strategy_

  - [x] 2.3 設定ファイルバックアップテスト
    - 正常系: バックアップファイルが作成される
    - 正常系: 更新成功時にバックアップが削除される
    - 異常系: 更新失敗時にバックアップから復元される
    - _Requirements: 11.5, Testing Strategy_

- [x] 3. Unit Tests: UrlValidationService
  - [x] 3.1 URL検証テスト
    - 正常系: 有効なHTTP URLを検証
    - 正常系: 有効なHTTPS URLを検証
    - 異常系: 不正なURL形式
    - エッジケース: 空文字列
    - エッジケース: 空白のみの文字列
    - エッジケース: 非常に長いURL
    - _Requirements: Testing Strategy_

  - [x] 3.2 URL配列検証テスト
    - 正常系: 複数の有効なURLを検証
    - 異常系: 空の配列
    - 異常系: 一部が不正なURL
    - エッジケース: 重複したURL
    - _Requirements: Testing Strategy_

- [x] 4. Unit Tests: Telegrafコントローラー
  - [x] 4.1 Docker操作テスト（モック使用）
    - 正常系: SIGHUPシグナル送信成功
    - 異常系: コンテナが存在しない
    - 異常系: Docker Socketにアクセスできない
    - 異常系: 権限不足
    - _Requirements: 11.2, 12.4, 12.5, Testing Strategy_

  - [x] 4.2 エラーハンドリングテスト
    - Docker CLIのエラー出力を適切に処理
    - ユーザーフレンドリーなエラーメッセージを返す
    - _Requirements: 11.5, 12.5, Testing Strategy_

- [x] 5. Property-Based Tests: Property 1（設定ファイル更新の原子性）
  - [x] 5.1 Property 1のテスト実装
    - **Property 1: 設定ファイル更新の原子性**
    - *For any* URL更新操作、設定ファイルへの書き込みが完了するまで、Telegrafへのリロード指示は送信されない
    - ランダムなURL配列を生成（1-100個）
    - 設定ファイル書き込みとSIGHUP送信の順序を検証
    - 最小100回実行
    - **Validates: Requirements 11.1, 11.2**
    - _Requirements: 11.1, 11.2, Testing Strategy_

- [x] 6. Property-Based Tests: Property 2（SIGHUPシグナルの確実な送信）
  - [x] 6.1 Property 2のテスト実装
    - **Property 2: SIGHUPシグナルの確実な送信**
    - *For any* 設定ファイル更新が成功した場合、TelegrafコンテナにSIGHUPシグナルが送信される
    - ランダムなURL配列を生成
    - 設定ファイル更新成功後、必ずSIGHUPが送信されることを検証
    - 最小100回実行
    - **Validates: Requirements 11.2, 12.4**
    - _Requirements: 11.2, 12.4, Testing Strategy_

- [x] 7. Property-Based Tests: Property 5（エラー時の設定ロールバック）
  - [x] 7.1 Property 5のテスト実装
    - **Property 5: エラー時の設定ロールバック**
    - *For any* 設定ファイル更新が失敗した場合、以前の設定ファイルが保持される
    - ランダムなURL配列を生成
    - 意図的にエラーを発生させる
    - 元の設定ファイルが保持されていることを検証
    - 最小100回実行
    - **Validates: Requirements 11.5**
    - _Requirements: 11.5, Testing Strategy_

- [x] 8. Integration Tests: Manager ↔ Telegraf
  - [x] 8.1 設定更新とリロードの統合テスト
    - テスト用のTelegrafコンテナを起動
    - 管理画面からURL更新を実行
    - 設定ファイルが更新されることを確認
    - SIGHUPシグナルが送信されることを確認
    - Telegrafが設定をリロードすることを確認
    - _Requirements: 11.1, 11.2, 11.3, Testing Strategy_

  - [x] 8.2 エラーハンドリングの統合テスト
    - Telegrafコンテナが停止している場合のエラー処理
    - 設定ファイルが不正な場合のエラー処理
    - ユーザーに適切なエラーメッセージが表示されることを確認
    - _Requirements: 11.5, Testing Strategy_

- [x] 9. Integration Tests: Manager ↔ Docker Socket
  - [x] 9.1 Docker Socket統合テスト
    - Docker Socketへのアクセスを確認
    - コンテナ一覧取得が成功することを確認
    - Telegrafコンテナの識別が成功することを確認
    - _Requirements: 12.1, 12.3, Testing Strategy_

  - [x] 9.2 Docker Socket利用不可時のテスト
    - Docker Socketがマウントされていない場合
    - Docker Socketへのアクセス権限がない場合
    - 適切なエラーハンドリングを確認
    - _Requirements: 12.5, Testing Strategy_

- [x] 10. End-to-End Tests: システム全体
  - [x] 10.1 E2Eテスト環境のセットアップ
    - Docker Composeでテスト環境を起動
    - 全サービスの起動を確認
    - ヘルスチェックの成功を確認
    - _Requirements: Testing Strategy_

  - [x] 10.2 E2Eテスト: 完全なワークフロー
    - システム全体を起動
    - 管理画面にログイン
    - URLを追加
    - Telegrafが新しいURLを監視開始することを確認
    - InfluxDBにデータが保存されることを確認
    - Grafanaでデータが表示されることを確認
    - _Requirements: 11.1, 11.2, 11.3, Testing Strategy_

  - [x] 10.3 E2Eテスト: エラーシナリオ
    - 不正なURLを追加した場合
    - Telegrafが停止している場合
    - InfluxDBが停止している場合
    - 各エラーケースで適切なエラーメッセージが表示されることを確認
    - _Requirements: 11.5, Testing Strategy_

- [x] 11. テストドキュメントの作成
  - [x] 11.1 テスト実行手順書の作成
    - `manager/tests/README.md`を作成
    - 各テストの実行方法を記載
    - テスト環境のセットアップ手順を記載
    - _Requirements: Testing Strategy_

  - [x] 11.2 テストカバレッジレポートの設定
    - `cargo-tarpaulin`または`cargo-llvm-cov`を設定
    - カバレッジレポート生成コマンドを追加
    - カバレッジ目標を設定（推奨: 80%以上）
    - _Requirements: Testing Strategy_

- [x] 12. CI/CDパイプラインへのテスト統合
  - [x] 12.1 GitHub Actions（または他のCI）設定
    - `.github/workflows/test.yml`を作成
    - プルリクエスト時に自動テスト実行
    - テスト失敗時にマージをブロック
    - _Requirements: Testing Strategy_

  - [x] 12.2 テストレポートの自動生成
    - テスト結果をCI/CDで表示
    - カバレッジレポートをCI/CDで表示
    - 失敗したテストの詳細を表示
    - _Requirements: Testing Strategy_

- [x] 13. 最終確認とドキュメント更新
  - [x] 13.1 全テストの実行確認
    - `cargo test`で全テストが成功することを確認
    - カバレッジが目標値を達成していることを確認
    - _Requirements: Testing Strategy_

  - [x] 13.2 READMEへのテスト情報追加
    - ルートREADME.mdにテスト実行方法を追加
    - テストカバレッジバッジを追加（オプション）
    - _Requirements: Testing Strategy_

## Notes

- Property-Based Testsは最小100回実行し、ランダムな入力で正確性を検証します
- Integration TestsとE2E Testsは実際のDockerコンテナを使用するため、実行時間が長くなります
- テストは段階的に実装し、各段階で動作確認を行います
- CI/CDパイプラインでの自動テスト実行を推奨します
