.PHONY: check rust-test go-test go-vet go-vuln go-lint ui-check format install-go-tools

check:
	./scripts/check.sh

rust-test:
	cargo test -p rustuary-core

go-test:
	cd services/api && go test ./...

go-vet:
	cd services/api && go vet ./...

go-vuln:
	cd services/api && govulncheck ./...

go-lint:
	cd services/api && golangci-lint run

ui-check:
	cd apps/workbench && pnpm check

format:
	cargo fmt --all
	cd services/api && gofmt -w .

install-go-tools:
	./scripts/install_go_tools.sh
