#!/bin/bash
set -e

# Dead or Alive System - SSL証明書更新スクリプト
# Let's Encrypt証明書を更新

echo "========================================="
echo "Dead or Alive System - SSL証明書更新"
echo "========================================="
echo ""

# 設定確認
if [ ! -f .env ]; then
    echo "❌ エラー: .envファイルが見つかりません"
    exit 1
fi

# 環境変数読み込み
source .env

# SSL_MODEチェック
if [ "${SSL_MODE:-letsencrypt}" != "letsencrypt" ]; then
    echo "⚠️  SSL_MODE が letsencrypt ではありません"
    echo "現在の設定: ${SSL_MODE}"
    echo ""
    echo "AWS ACMを使用している場合、このスクリプトは不要です"
    exit 0
fi

# ドメイン設定確認
DOMAIN_GRAFANA="${DOMAIN_GRAFANA:-doa.com}"
DOMAIN_MANAGER="${DOMAIN_MANAGER:-admin.doa.com}"

echo "📋 設定:"
echo "  - Grafanaドメイン: $DOMAIN_GRAFANA"
echo "  - 管理画面ドメイン: $DOMAIN_MANAGER"
echo "  - メールアドレス: ${LETSENCRYPT_EMAIL}"
echo ""

# メールアドレスチェック
if [ -z "${LETSENCRYPT_EMAIL}" ]; then
    echo "❌ エラー: LETSENCRYPT_EMAILが設定されていません"
    echo "💡 .envファイルでLETSENCRYPT_EMAILを設定してください"
    exit 1
fi

# Certbotディレクトリ作成
mkdir -p nginx/certbot

# 証明書更新
echo "🔐 証明書を更新しています..."
docker run --rm \
    -v $(pwd)/nginx/ssl:/etc/letsencrypt \
    -v $(pwd)/nginx/certbot:/var/www/certbot \
    certbot/certbot renew \
    --webroot \
    --webroot-path=/var/www/certbot \
    --email "${LETSENCRYPT_EMAIL}" \
    --agree-tos \
    --no-eff-email

echo "✅ 証明書更新完了"
echo ""

# Nginx設定リロード
echo "🔄 Nginx設定をリロードしています..."
if docker-compose ps nginx | grep -q "Up"; then
    docker-compose exec nginx nginx -s reload
    echo "✅ Nginxリロード完了"
else
    echo "⚠️  Nginxコンテナが起動していません"
    echo "💡 docker-compose up -d nginx を実行してください"
fi
echo ""

# 証明書有効期限確認
echo "📅 証明書有効期限:"
if [ -f "nginx/ssl/live/$DOMAIN_GRAFANA/cert.pem" ]; then
    openssl x509 -in "nginx/ssl/live/$DOMAIN_GRAFANA/cert.pem" -noout -dates
fi
echo ""

echo "========================================="
echo "✅ SSL証明書更新完了"
echo "========================================="
echo ""
echo "💡 自動更新を設定するには、crontabに以下を追加:"
echo "   0 3 * * * cd $(pwd) && ./scripts/ssl-renew.sh >> logs/ssl-renew.log 2>&1"
