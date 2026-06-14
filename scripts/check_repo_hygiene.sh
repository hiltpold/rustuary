#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "AGENTS.md"
  "CHANGELOG.md"
  "CONTRIBUTING.md"
  "RELEASE.md"
  "SECURITY.md"
  "GOVERNANCE.md"
  "ROADMAP.md"
  "rust-toolchain.toml"
  ".mise.toml"
  ".tool-versions"
  ".node-version"
  ".python-version"
  ".devcontainer/devcontainer.json"
  ".vscode/extensions.json"
  ".github/CODEOWNERS"
  ".github/pull_request_template.md"
  ".github/ISSUE_TEMPLATE/feature.yml"
  ".github/ISSUE_TEMPLATE/bug.yml"
  ".github/ISSUE_TEMPLATE/actuarial-method.yml"
  "docs/ai/task-brief-template.md"
  "docs/ai/implementation-report-template.md"
  "docs/adr/README.md"
  "docs/templates/adr-template.md"
  "docs/standards/engineering-principles.md"
  "docs/standards/change-management.md"
  "docs/standards/testing-strategy.md"
  "docs/actuarial/model-governance.md"
  "contracts/DATA_CONTRACTS.md"
  ".agents/skills/go-service/SKILL.md"
  ".agents/skills/sveltekit-ui/SKILL.md"
  ".codex/config.example.toml"
  ".mcp/svelte-remote.json"
  ".mcp/svelte-local.json"
  "docs/ai/mcp-setup.md"
  "docs/adr/0005-use-scoped-ai-tooling-for-go-and-sveltekit.md"
  ".golangci.yml"
)

missing=0
for file in "${required_files[@]}"; do
  if [ ! -f "$file" ]; then
    echo "missing required repo governance file: $file" >&2
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  exit 1
fi

if ! grep -q "Documentation update rule" CONTRIBUTING.md; then
  echo "CONTRIBUTING.md must include the documentation update rule" >&2
  exit 1
fi

if ! grep -q "CHANGELOG.md" .github/pull_request_template.md; then
  echo "PR template must remind contributors to update CHANGELOG.md" >&2
  exit 1
fi

if ! grep -q "engineering-principles" AGENTS.md; then
  echo "AGENTS.md must reference engineering principles" >&2
  exit 1
fi

if ! grep -q "model-governance" AGENTS.md docs/standards/definition_of_done.md; then
  echo "agent/DOD guidance must reference model governance" >&2
  exit 1
fi

if ! grep -q "Task brief" docs/ai/task-brief-template.md .github/pull_request_template.md; then
  echo "task brief template must be discoverable from PR workflow" >&2
  exit 1
fi

if ! grep -q "Implementation report" docs/ai/implementation-report-template.md .github/pull_request_template.md; then
  echo "implementation report template must be discoverable from PR workflow" >&2
  exit 1
fi

if ! grep -q "Svelte MCP" docs/ai/mcp-setup.md .agents/skills/sveltekit-ui/SKILL.md; then
  echo "Svelte MCP setup must be documented" >&2
  exit 1
fi

if ! grep -q "govulncheck" .agents/skills/go-service/SKILL.md scripts/check.sh; then
  echo "Go agent/check guidance must reference govulncheck" >&2
  exit 1
fi

if ! grep -q "cargo clippy" scripts/check.sh .github/workflows/ci.yml; then
  echo "Rust checks must include cargo clippy" >&2
  exit 1
fi

if ! grep -q "cargo doc" scripts/check.sh .github/workflows/ci.yml; then
  echo "Rust checks must include cargo doc" >&2
  exit 1
fi

echo "repo hygiene files present"
