#!/bin/bash
# JSON output + jq for scripting and automation
# Requires: jq (brew install jq)

set -e

echo "=== JSON Scripting Examples ==="
echo ""

# List model names
echo "→ Model names:"
aix models list --limit 5 --json | jq -r '.results[].name'
echo ""

# Count total available models
echo "→ Total models:"
aix models list --limit 1 --json | jq '.total'
echo ""

# List agent names and their tool counts
echo "→ Agents with tool counts:"
aix agents list --limit 5 --json | jq -r '.results[] | "\(.name) — \(.tools | length) tools"'
echo ""

# Get all integration names
echo "→ Integration names:"
aix integrations list --limit 10 --json | jq -r '.results[].name'
echo ""

# Find models that support a specific function
echo "→ Translation models:"
aix models list --query "translation" --json | jq -r '.results[] | "\(.name) (\(.vendor.name // "unknown"))"'
echo ""

echo "Done!"
