#!/bin/bash

# Dead or Alive System - ヘルスチェックスクリプト
# 全サービスの健全性を確認

echo "========================================="
echo "Dead or Alive System - ヘルスチェック"
echo "========================================="
echo ""

HEALTH_OK=true

# Nginx
echo "🌐 Nginx:"
if curl -f http://localhost/health >/dev/null 2>&1; then
    echo "  ✅ OK"
else
    echo "  ❌ NG"
    HEALTH_OK=false
fi
echo ""

# Manager
echo "⚙️  Manager:"
if curl -f http://localhost:8081/health >/dev/null 2>&1; then
    echo "  ✅ OK"
else
    echo "  ❌ NG"
    HEALTH_OK=false
fi
echo ""

# Telegraf
echo "📡 Telegraf:"
if docker-compose ps telegraf | grep -q "Up"; then
    echo "  ✅ OK (Running)"
else
    echo "  ❌ NG (Not Running)"
    HEALTH_OK=false
fi
echo ""

# InfluxDB
echo "📊 InfluxDB:"
if docker-compose exec -T influxdb influx ping >/dev/null 2>&1; then
    echo "  ✅ OK"
else
    echo "  ❌ NG"
    HEALTH_OK=false
fi
echo ""

# Grafana
echo "📈 Grafana:"
if curl -f http://localhost:3000/api/health >/dev/null 2>&1; then
    echo "  ✅ OK"
else
    echo "  ❌ NG"
    HEALTH_OK=false
fi
echo ""

# Docker Compose サービス状態
echo "🐳 Docker Compose サービス:"
docker-compose ps
echo ""

# リソース使用状況
echo "💻 リソース使用状況:"
docker stats --no-stream $(docker-compose ps -q) 2>/dev/null || echo "  （取得できませんでした）"
echo ""

# 結果
echo "========================================="
if [ "$HEALTH_OK" = true ]; then
    echo "✅ 全サービス正常"
    echo "========================================="
    exit 0
else
    echo "⚠️  一部サービスに問題があります"
    echo "========================================="
    echo ""
    echo "ログを確認してください:"
    echo "  docker-compose logs"
    exit 1
fi
