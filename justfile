run:
  RUST_LOG=debug cargo run

dev:
  RUST_LOG=debug cargo watch -x run

build: 
  cargo sqlx prepare --workspace
  cargo build

build-frontend:
  cd frontend && npm run build

migrate:
  sqlx migrate run

remigrate:
  sqlx database drop && sqlx database create && sqlx migrate run

infra:
  docker compose up -d

encode-secrets:
  kubeseal --controller-name=sealed-secrets --controller-namespace=sealed-secrets --context default \
  --format yaml < secrets.yaml > sealed-secrets.yaml

apply-secrets:
  kubectl --context default apply -f sealed-secrets.yaml

deploy-from-local:
  docker build . -t hub.storeinvoice.app/mithrilforge:latest -t latest
  docker push hub.storeinvoice.app/mithrilforge:latest
  helm upgrade --install mithrilforge ./.helm/ --set image.repository=hub.storeinvoice.app/mithrilforge --set image.tag=latest
