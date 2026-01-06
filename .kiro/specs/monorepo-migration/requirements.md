# Requirements Document

## Introduction

このドキュメントは、現在の`kiro_test`リポジトリと`../deadoralive`リポジトリを統合し、`deadoralive-system`という新しいモノレポ構成に再構築するための要件を定義します。

**新しいディレクトリの作成場所**: 現在のワークスペースの親ディレクトリ（`../deadoralive-system/`）に作成されます。

```
現在の構造:
parent-directory/
├── kiro_test/           # 現在のワークスペース（管理系）
└── deadoralive/         # 監視系

移行後の構造:
parent-directory/
├── kiro_test/           # 旧リポジトリ（保持）
├── deadoralive/         # 旧リポジトリ（保持）
└── deadoralive-system/  # 新しいモノレポ（新規作成）
    ├── manager/         # kiro_testから移行
    ├── monitoring/      # deadoraliveから移行
    ├── docker-compose.yml
    └── ...
```

## Glossary

- **Monorepo**: 複数のプロジェクトを1つのリポジトリで管理する構成
- **Manager System**: Telegraf設定を管理するRustアプリケーション（旧kiro_test）
- **Monitoring System**: Telegraf、InfluxDB、Grafanaによる監視システム（旧deadoralive）
- **Docker Compose**: 複数のDockerコンテナを定義・実行するツール
- **Migration**: 既存のリポジトリから新しいリポジトリへの移行

## Requirements

### Requirement 1

**User Story:** 開発者として、監視系と管理系を1つのリポジトリで管理したいので、モノレポ構成に移行する必要がある

#### Acceptance Criteria

1. WHEN 新しいリポジトリ構造を作成するとき THEN `deadoralive-system`という名前のディレクトリが作成される
2. WHEN ディレクトリ構造を確認するとき THEN `monitoring/`と`manager/`のサブディレクトリが存在する
3. WHEN ルートディレクトリを確認するとき THEN 統合された`docker-compose.yml`が存在する
4. WHEN ドキュメントを確認するとき THEN システム全体を説明する`README.md`が存在する

### Requirement 2

**User Story:** 開発者として、既存の管理系コードを新しい構造に移行したいので、`manager/`ディレクトリに適切に配置する必要がある

#### Acceptance Criteria

1. WHEN 管理系のソースコードを移行するとき THEN `manager/src/`にRustコードが配置される
2. WHEN 管理系の設定ファイルを移行するとき THEN `manager/config/`に設定ファイルが配置される
3. WHEN 管理系のアセットを移行するとき THEN `manager/assets/`にHTMLテンプレートが配置される
4. WHEN Dockerfileを確認するとき THEN `manager/Dockerfile`が存在し、マルチステージビルドを使用している
5. WHEN 不要なファイルを確認するとき THEN 開発用ドキュメント（guide/, docs/）は移行されない

### Requirement 3

**User Story:** 開発者として、既存の監視系設定を新しい構造に移行したいので、`monitoring/`ディレクトリに適切に配置する必要がある

#### Acceptance Criteria

1. WHEN Telegraf設定を移行するとき THEN `monitoring/telegraf/telegraf.conf`が存在する
2. WHEN InfluxDB設定を移行するとき THEN `monitoring/influxdb/`ディレクトリが存在する
3. WHEN Grafana設定を移行するとき THEN `monitoring/grafana/`ディレクトリが存在する
4. WHEN 設定ファイルを確認するとき THEN 全ての設定ファイルが適切なサブディレクトリに配置されている

### Requirement 4

**User Story:** 開発者として、統合されたDocker Compose構成を使用したいので、ルートに統合された設定ファイルが必要である

#### Acceptance Criteria

1. WHEN Docker Compose設定を確認するとき THEN ルートに`docker-compose.yml`が存在する
2. WHEN サービス定義を確認するとき THEN nginx、manager、telegraf、influxdb、grafanaの5つのサービスが定義されている
3. WHEN ボリューム定義を確認するとき THEN 設定ファイル共有用のボリュームが定義されている
4. WHEN ネットワーク定義を確認するとき THEN frontendとbackendの2つのネットワークが定義されている
5. WHEN ヘルスチェックを確認するとき THEN 各サービスにヘルスチェックが定義されている

### Requirement 5

**User Story:** 開発者として、Nginxリバースプロキシを使用したいので、適切な設定ファイルが必要である

#### Acceptance Criteria

1. WHEN Nginx設定を確認するとき THEN `nginx/nginx.conf`が存在する
2. WHEN ドメインルーティングを確認するとき THEN `doa.com`がgrafanaにルーティングされる
3. WHEN ドメインルーティングを確認するとき THEN `admin.doa.com`がmanagerにルーティングされる
4. WHEN SSL設定を確認するとき THEN `nginx/ssl/`ディレクトリが存在する
5. WHEN HTTPリクエストを受信するとき THEN HTTPSにリダイレクトされる

### Requirement 6

**User Story:** 開発者として、運用を簡素化したいので、Makefileで一般的なコマンドを実行できる必要がある

#### Acceptance Criteria

1. WHEN Makefileを確認するとき THEN ルートに`Makefile`が存在する
2. WHEN `make up`を実行するとき THEN 全てのサービスが起動する
3. WHEN `make down`を実行するとき THEN 全てのサービスが停止する
4. WHEN `make logs`を実行するとき THEN 全てのサービスのログが表示される
5. WHEN `make help`を実行するとき THEN 利用可能なコマンドの一覧が表示される

### Requirement 7

**User Story:** 開発者として、環境変数を管理したいので、テンプレートファイルが必要である

#### Acceptance Criteria

1. WHEN 環境変数テンプレートを確認するとき THEN `.env.example`が存在する
2. WHEN テンプレートを確認するとき THEN JWT_SECRET、INFLUXDB_PASSWORD、GRAFANA_PASSWORDが定義されている
3. WHEN `.gitignore`を確認するとき THEN `.env`ファイルが除外されている
4. WHEN 環境変数を設定するとき THEN `.env.example`をコピーして`.env`を作成できる

### Requirement 8

**User Story:** 開発者として、システムのドキュメントを参照したいので、包括的なREADMEが必要である

#### Acceptance Criteria

1. WHEN READMEを確認するとき THEN システムの概要が記載されている
2. WHEN READMEを確認するとき THEN クイックスタート手順が記載されている
3. WHEN READMEを確認するとき THEN ディレクトリ構造の説明が記載されている
4. WHEN READMEを確認するとき THEN デプロイ手順へのリンクが記載されている
5. WHEN READMEを確認するとき THEN トラブルシューティングへのリンクが記載されている

### Requirement 9

**User Story:** 開発者として、詳細なドキュメントを参照したいので、`docs/`ディレクトリに整理されたドキュメントが必要である

#### Acceptance Criteria

1. WHEN ドキュメントディレクトリを確認するとき THEN `docs/`ディレクトリが存在する
2. WHEN デプロイドキュメントを確認するとき THEN `docs/DEPLOYMENT.md`が存在する
3. WHEN 運用ドキュメントを確認するとき THEN `docs/OPERATION.md`が存在する
4. WHEN トラブルシューティングドキュメントを確認するとき THEN `docs/TROUBLESHOOTING.md`が存在する
5. WHEN アーキテクチャドキュメントを確認するとき THEN `docs/ARCHITECTURE.md`が存在する

### Requirement 10

**User Story:** 開発者として、運用スクリプトを使用したいので、`scripts/`ディレクトリに整理されたスクリプトが必要である

#### Acceptance Criteria

1. WHEN スクリプトディレクトリを確認するとき THEN `scripts/`ディレクトリが存在する
2. WHEN デプロイスクリプトを確認するとき THEN `scripts/deploy.sh`が存在する
3. WHEN バックアップスクリプトを確認するとき THEN `scripts/backup.sh`が存在する
4. WHEN リストアスクリプトを確認するとき THEN `scripts/restore.sh`が存在する
5. WHEN ヘルスチェックスクリプトを確認するとき THEN `scripts/health-check.sh`が存在する

### Requirement 11

**User Story:** 運用者として、管理画面でURL設定を更新したときに、Telegrafに自動的に反映させたいので、設定リロード機能が必要である

#### Acceptance Criteria

1. WHEN 管理画面でURLを更新するとき THEN 共有ボリュームの`telegraf.conf`が更新される
2. WHEN 設定ファイルが更新されたとき THEN TelegrafコンテナにSIGHUPシグナルが送信される
3. WHEN SIGHUPシグナルを受信するとき THEN Telegrafが設定をリロードする（再起動不要）
4. WHEN 設定リロードが完了するとき THEN ユーザーに成功メッセージが表示される
5. WHEN 設定リロードが失敗するとき THEN ユーザーにエラーメッセージが表示される

### Requirement 12

**User Story:** 開発者として、Telegrafコンテナを制御したいので、管理系コンテナからDocker Socketにアクセスできる必要がある

#### Acceptance Criteria

1. WHEN Docker Compose設定を確認するとき THEN managerサービスに`/var/run/docker.sock`がマウントされている
2. WHEN managerコンテナを確認するとき THEN Docker CLIがインストールされている
3. WHEN Telegrafコンテナを確認するとき THEN `com.deadoralive.service=telegraf`ラベルが付与されている
4. WHEN 管理系コードを確認するとき THEN `docker kill --signal=SIGHUP telegraf`コマンドを実行する機能が存在する
5. WHEN エラーハンドリングを確認するとき THEN Docker操作の失敗を適切に処理する

### Requirement 13

**User Story:** 開発者として、Git管理を適切に行いたいので、適切な`.gitignore`ファイルが必要である

#### Acceptance Criteria

1. WHEN `.gitignore`を確認するとき THEN ルートに`.gitignore`が存在する
2. WHEN 除外ファイルを確認するとき THEN `.env`ファイルが除外されている
3. WHEN 除外ファイルを確認するとき THEN `target/`ディレクトリが除外されている
4. WHEN 除外ファイルを確認するとき THEN `db.sqlite`ファイルが除外されている
5. WHEN 除外ファイルを確認するとき THEN SSL証明書ファイルが除外されている
