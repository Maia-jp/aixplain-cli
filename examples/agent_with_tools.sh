#!/bin/bash
# Create agents with tools and sub-agents attached
# Make sure .env is configured with your API key before running.
#
# This example shows the full flow:
#   1. Find tools from the platform
#   2. Create an agent with tools attached
#   3. Run the agent (it can use its tools)
#   4. Add more tools later
#   5. Compose agents with sub-agents

set -e

echo "=== Agent + Tools Workflow ==="
echo ""

# ── Step 1: Discover available tools ─────────────────

echo "→ Available tools:"
aix tools list --limit 5
echo ""

# Grab the first tool ID automatically
TOOL_ID=$(aix tools list --limit 1 --json | python3 -c "import sys,json; print(json.load(sys.stdin)['results'][0]['id'])")
echo "  Using tool: $TOOL_ID"
echo ""

# ── Step 2: Create agent with a tool attached ────────

echo "→ Creating agent with tool..."
AGENT_JSON=$(aix agents create \
  --name "Tool-Equipped Agent" \
  --instructions "You are an assistant with access to tools. Use them when needed." \
  --tool "$TOOL_ID" \
  --json)

AGENT_ID=$(echo "$AGENT_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "  Created agent: $AGENT_ID"
echo ""

# ── Step 3: Verify tools are attached ────────────────

echo "→ Agent details (check tools):"
aix agents get "$AGENT_ID"
echo ""

# ── Step 4: Add another tool to the agent ────────────

# Get a second tool
TOOL_ID_2=$(aix tools list --limit 2 --json | python3 -c "import sys,json; r=json.load(sys.stdin)['results']; print(r[1]['id'] if len(r)>1 else r[0]['id'])")
echo "→ Adding second tool: $TOOL_ID_2"
aix agents update "$AGENT_ID" --add-tool "$TOOL_ID_2"
echo ""

echo "→ Updated agent details:"
aix agents get "$AGENT_ID"
echo ""

# ── Step 5: Replace tools entirely ───────────────────

echo "→ Replacing all tools with just one:"
aix agents update "$AGENT_ID" --tool "$TOOL_ID"
echo ""

# ── Step 6: Create a sub-agent and orchestrate ───────

echo "→ Creating sub-agent..."
SUB_JSON=$(aix agents create \
  --name "Sub-Agent (Worker)" \
  --instructions "You are a worker agent that handles specific tasks." \
  --json)

SUB_ID=$(echo "$SUB_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "  Created sub-agent: $SUB_ID"
echo ""

echo "→ Creating orchestrator with sub-agent..."
ORCH_JSON=$(aix agents create \
  --name "Orchestrator Agent" \
  --instructions "You coordinate sub-agents to solve complex tasks." \
  --subagent "$SUB_ID" \
  --tool "$TOOL_ID" \
  --json)

ORCH_ID=$(echo "$ORCH_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "  Created orchestrator: $ORCH_ID"
echo ""

echo "→ Orchestrator details:"
aix agents get "$ORCH_ID"
echo ""

# ── Cleanup ──────────────────────────────────────────

echo "→ Cleaning up..."
aix agents delete "$AGENT_ID" --force
aix agents delete "$SUB_ID" --force
aix agents delete "$ORCH_ID" --force
echo "  All agents deleted."
echo ""

echo "=== Summary ==="
echo ""
echo "Key commands used:"
echo "  aix agents create --name '...' --tool <ID>              # Create with tools"
echo "  aix agents create --name '...' --tool <ID> --tool <ID>  # Multiple tools"
echo "  aix agents update <ID> --add-tool <TOOL_ID>             # Add tool later"
echo "  aix agents update <ID> --tool <ID>                      # Replace all tools"
echo "  aix agents create --name '...' --subagent <AGENT_ID>    # Multi-agent"
echo ""
echo "Done!"
