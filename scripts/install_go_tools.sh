#!/usr/bin/env bash
set -euo pipefail

go install golang.org/x/tools/gopls@latest
go install golang.org/x/vuln/cmd/govulncheck@latest

echo "Installed gopls and govulncheck into $(go env GOPATH)/bin"
echo "Install golangci-lint from https://golangci-lint.run/docs/welcome/install/ if needed."
