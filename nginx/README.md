# Nginx Configuration

このディレクトリには、Dead or Alive Systemのリバースプロキシ設定が含まれています。

## 設定ファイル

### nginx.conf（Let's Encrypt用）

EC2単体で運用する場合に使用します。NginxでSSL/TLS終端を行います。

**特徴:**
- Let's Encryptによる証明書管理
- HTTPSへの自動リダイレクト
- SSL/TLS設定（TLS 1.2/1.3）
- HSTS（HTTP Strict Transport Security）

**使用方法:**
```yaml
# docker-compose.ymlで指定
services:
  nginx:
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
```

### nginx-aws.conf（AWS ALB + ACM用）

AWS環境でALB + ACMを使用する場合に使用します。NginxはHTTPのみで動作します。

**特徴:**
- HTTPのみ（SSL/TLS終端はALBで実施）
- Real IP取得（X-Forwarded-Forヘッダー）
- ALB経由のアクセスに最適化

**使用方法:**
```yaml
# docker-compose.ymlで指定
services:
  nginx:
    volumes:
      - ./nginx/nginx-aws.conf:/etc/nginx/nginx.conf:ro
```

## ドメインルーティング

両方の設定ファイルで以下のルーティングを提供します：

- `doa.com` → Grafana (Port 3000)
- `admin.doa.com` → Manager (Port 8081)

## セキュリティ機能

### セキュリティヘッダー

**全サービス共通:**
- X-Frame-Options: クリックジャッキング防止
- X-Content-Type-Options: MIMEスニッフィング防止
- X-XSS-Protection: XSS攻撃防止

**管理画面（admin.doa.com）:**
- Content-Security-Policy: より厳格なCSP
- X-Frame-Options: DENY（より厳格）

**Let's Encrypt版のみ:**
- Strict-Transport-Security (HSTS): HTTPS強制

### Rate Limiting

- 一般ユーザー（Grafana）: 10リクエスト/秒、バースト20
- 管理画面: 5リクエスト/秒、バースト10

### IP制限（オプション）

管理画面へのアクセスを特定IPに制限できます：

```nginx
# nginx.confまたはnginx-aws.confで設定
location / {
    allow 203.0.113.0/24;  # 許可するIPレンジ
    deny all;
    # ...
}
```

## パフォーマンス最適化

### Gzip圧縮

テキストベースのコンテンツを圧縮してデータ転送量を削減：
- 対象: HTML, CSS, JavaScript, JSON, XML
- 最小サイズ: 1KB以上

### HTTP/2（Let's Encrypt版のみ）

- 多重化による高速化
- ヘッダー圧縮

### プロキシバッファリング

- バックエンドの負荷軽減
- レスポンス時間の改善

## SSL証明書

### Let's Encrypt使用時

証明書は`ssl/`ディレクトリに配置します：

```
ssl/
└── live/
    ├── doa.com/
    │   ├── fullchain.pem
    │   └── privkey.pem
    └── admin.doa.com/
        ├── fullchain.pem
        └── privkey.pem
```

**証明書取得:**
```bash
# Certbotを使用
certbot certonly --webroot -w /var/www/certbot \
  -d doa.com -d admin.doa.com
```

### AWS ACM使用時

証明書管理はAWS Certificate Managerで行います。Nginxでの証明書設定は不要です。

## ヘルスチェック

両方の設定ファイルで`/health`エンドポイントを提供：

```bash
curl http://localhost/health
# => healthy
```

## トラブルシューティング

### 設定ファイルの検証

```bash
docker-compose exec nginx nginx -t
```

### ログの確認

```bash
docker-compose logs nginx
```

### 設定のリロード

```bash
docker-compose exec nginx nginx -s reload
```

## 推奨構成

| 環境 | 推奨設定 | 理由 |
|------|---------|------|
| AWS | nginx-aws.conf + ALB + ACM | 証明書自動更新、スケーラビリティ |
| EC2単体 | nginx.conf + Let's Encrypt | シンプル、コスト削減 |
| 開発環境 | nginx-aws.conf（HTTP） | SSL不要、セットアップ簡単 |
