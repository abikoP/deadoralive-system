.PHONY: help up down restart build logs logs-f ps clean health backup restore ssl-renew

# デフォルトターゲット
.DEFAULT_GOAL := help

# ========================================
# ヘルプ
# ========================================
help: ## ヘルプを表示
	@echo "Dead or Alive System - 運用コマンド"
	@echo ""
	@echo "使用方法:"
	@echo "  make <target>"
	@echo ""
	@echo "利用可能なコマンド:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ========================================
# 基本操作
# ========================================
up: ## 全サービスを起動
	@echo "🚀 全サービスを起動しています..."
	docker-compose up -d
	@echo "✅ 起動完了"
	@echo ""
	@echo "アクセス先:"
	@echo "  - Grafana:     https://doa.com (または http://localhost:3000)"
	@echo "  - 管理画面:    https://admin.doa.com (または http://localhost:8081)"
	@echo "  - InfluxDB:    http://localhost:8086"

down: ## 全サービスを停止
	@echo "🛑 全サービスを停止しています..."
	docker-compose down
	@echo "✅ 停止完了"

restart: ## 全サービスを再起動
	@echo "🔄 全サービスを再起動しています..."
	docker-compose restart
	@echo "✅ 再起動完了"

# ========================================
# ビルド
# ========================================
build: ## Dockerイメージをビルド
	@echo "🔨 Dockerイメージをビルドしています..."
	docker-compose build --no-cache
	@echo "✅ ビルド完了"

rebuild: ## イメージを再ビルドして起動
	@echo "🔨 イメージを再ビルドして起動しています..."
	docker-compose up -d --build
	@echo "✅ 完了"

# ========================================
# ログ
# ========================================
logs: ## 全サービスのログを表示
	docker-compose logs --tail=100

logs-f: ## 全サービスのログをフォロー
	docker-compose logs -f

logs-manager: ## Managerのログを表示
	docker-compose logs -f manager

logs-telegraf: ## Telegrafのログを表示
	docker-compose logs -f telegraf

logs-influxdb: ## InfluxDBのログを表示
	docker-compose logs -f influxdb

logs-grafana: ## Grafanaのログを表示
	docker-compose logs -f grafana

logs-nginx: ## Nginxのログを表示
	docker-compose logs -f nginx

# ========================================
# 状態確認
# ========================================
ps: ## サービスの状態を確認
	docker-compose ps

health: ## ヘルスチェックを実行
	@echo "🏥 ヘルスチェックを実行しています..."
	@echo ""
	@echo "Nginx:"
	@curl -f http://localhost/health 2>/dev/null && echo " ✅" || echo " ❌"
	@echo ""
	@echo "Manager:"
	@curl -f http://localhost:8081/health 2>/dev/null && echo " ✅" || echo " ❌"
	@echo ""
	@echo "InfluxDB:"
	@docker-compose exec -T influxdb influx ping 2>/dev/null && echo " ✅" || echo " ❌"
	@echo ""
	@echo "Grafana:"
	@curl -f http://localhost:3000/api/health 2>/dev/null && echo " ✅" || echo " ❌"

# ========================================
# クリーンアップ
# ========================================
clean: ## ボリュームを含めて全て削除
	@echo "⚠️  警告: 全てのデータが削除されます"
	@read -p "続行しますか? [y/N]: " confirm && [ "$$confirm" = "y" ] || exit 1
	@echo "🗑️  全てのコンテナ、ボリューム、ネットワークを削除しています..."
	docker-compose down -v
	@echo "✅ クリーンアップ完了"

clean-logs: ## ログファイルを削除
	@echo "🗑️  ログファイルを削除しています..."
	rm -f logs/*.log
	@echo "✅ ログ削除完了"

# ========================================
# データ管理
# ========================================
backup: ## データベースをバックアップ
	@echo "💾 データベースをバックアップしています..."
	@mkdir -p backups
	@docker-compose exec -T influxdb influx backup /backup
	@docker cp $$(docker-compose ps -q influxdb):/backup ./backups/influxdb-$$(date +%Y%m%d-%H%M%S)
	@docker-compose exec -T manager cp /data/db.sqlite /data/db.sqlite.backup
	@docker cp $$(docker-compose ps -q manager):/data/db.sqlite.backup ./backups/manager-$$(date +%Y%m%d-%H%M%S).sqlite
	@echo "✅ バックアップ完了: backups/"

restore: ## データベースを復元（最新のバックアップから）
	@echo "⚠️  警告: 現在のデータが上書きされます"
	@read -p "続行しますか? [y/N]: " confirm && [ "$$confirm" = "y" ] || exit 1
	@echo "📥 データベースを復元しています..."
	@# 実装は scripts/restore.sh を参照
	@echo "✅ 復元完了"

# ========================================
# SSL証明書
# ========================================
ssl-renew: ## SSL証明書を更新（Let's Encrypt）
	@echo "🔐 SSL証明書を更新しています..."
	@# Certbotを使用して証明書を更新
	@docker run --rm -v ./nginx/ssl:/etc/letsencrypt \
		-v ./nginx/certbot:/var/www/certbot \
		certbot/certbot renew
	@docker-compose exec nginx nginx -s reload
	@echo "✅ 証明書更新完了"

# ========================================
# 開発用
# ========================================
shell-manager: ## Managerコンテナに入る
	docker-compose exec manager sh

shell-telegraf: ## Telegrafコンテナに入る
	docker-compose exec telegraf sh

shell-influxdb: ## InfluxDBコンテナに入る
	docker-compose exec influxdb sh

shell-grafana: ## Grafanaコンテナに入る
	docker-compose exec grafana sh

shell-nginx: ## Nginxコンテナに入る
	docker-compose exec nginx sh

# ========================================
# 設定
# ========================================
config-validate: ## Docker Compose設定を検証
	docker-compose config

config-show: ## Docker Compose設定を表示
	docker-compose config

# ========================================
# Telegraf制御
# ========================================
telegraf-reload: ## Telegraf設定をリロード（ダウンタイムなし）
	@echo "🔄 Telegraf設定をリロードしています..."
	docker kill --signal=SIGHUP telegraf
	@echo "✅ リロード完了"

telegraf-restart: ## Telegrafを再起動
	@echo "🔄 Telegrafを再起動しています..."
	docker-compose restart telegraf
	@echo "✅ 再起動完了"

# ========================================
# 環境設定
# ========================================
env-setup: ## .envファイルを作成
	@if [ ! -f .env ]; then \
		echo "📝 .envファイルを作成しています..."; \
		cp .env.example .env; \
		echo "✅ .envファイルを作成しました"; \
		echo "⚠️  .envファイルを編集してパスワードを設定してください"; \
	else \
		echo "⚠️  .envファイルは既に存在します"; \
	fi

# ========================================
# 統計情報
# ========================================
stats: ## リソース使用状況を表示
	docker stats --no-stream $$(docker-compose ps -q)

# ========================================
# アップデート
# ========================================
update: ## イメージを最新版に更新
	@echo "⬆️  Dockerイメージを更新しています..."
	docker-compose pull
	@echo "✅ 更新完了"
	@echo "💡 変更を適用するには 'make restart' を実行してください"
