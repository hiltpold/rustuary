check:
    ./scripts/check.sh

rust-test:
    cargo test -p rustuary-core

rust-fmt-check:
    cargo fmt --all -- --check

api-test:
    cd services/api && go test ./...


api-vet:
    cd services/api && go vet ./...

api-lint:
    cd services/api && golangci-lint run

api-vuln:
    cd services/api && govulncheck ./...

install-go-tools:
    ./scripts/install_go_tools.sh
