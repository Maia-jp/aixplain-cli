#!/bin/bash
# Basic CLI usage examples for aiXplain CLI
# Make sure .env is configured with your API key before running.

set -e

echo "=== Models ==="
echo ""

# List first 5 models
echo "→ aix models list --limit 5"
aix models list --limit 5
echo ""

# Search for translation models
echo "→ aix models list --query translation"
aix models list --query "translation"
echo ""

# Get model details (replace with a real ID from the list above)
# echo "→ aix models get <MODEL_ID>"
# aix models get 66aa869f6eb56342c26057e1
# echo ""

echo "=== Agents ==="
echo ""

# List agents
echo "→ aix agents list --limit 5"
aix agents list --limit 5
echo ""

echo "=== Tools ==="
echo ""

# List tools
echo "→ aix tools list --limit 5"
aix tools list --limit 5
echo ""

echo "=== Integrations ==="
echo ""

# List integrations
echo "→ aix integrations list --limit 5"
aix integrations list --limit 5
echo ""

echo "=== API Key Usage ==="
echo ""

# Check current key usage
echo "→ aix api-keys usage"
aix api-keys usage
echo ""

echo "Done!"
