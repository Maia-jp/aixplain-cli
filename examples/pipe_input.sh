#!/bin/bash
# Piping input from files and other commands
# Make sure .env is configured and replace MODEL_ID/AGENT_ID with real IDs.

set -e

MODEL_ID="${1:?Usage: $0 <MODEL_ID> [AGENT_ID]}"
AGENT_ID="${2:-}"

echo "=== Pipe Input Examples ==="
echo ""

# Pipe text to a model
echo "→ Pipe string to model:"
echo "What is the capital of France?" | aix models run "$MODEL_ID" --stdin
echo ""

# Pipe file content to a model
echo "→ Pipe file to model:"
echo "Explain this code: $(cat examples/basic_cli.sh | head -5)" | aix models run "$MODEL_ID" --stdin
echo ""

# Chain commands
if [ -n "$AGENT_ID" ]; then
  echo "→ Pipe to agent:"
  echo "Summarize this in one sentence: Rust is a systems programming language focused on safety, speed, and concurrency." \
    | aix agents run "$AGENT_ID" --stdin --timeout 120
  echo ""
fi

echo "Done!"
