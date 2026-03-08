#!/bin/bash
# Agent lifecycle: create (with tools) → run → update → delete
# Make sure .env is configured with your API key before running.

set -e

echo "=== Agent Workflow ==="
echo ""

# 1. Create a simple agent
echo "→ Creating agent..."
AGENT_JSON=$(aix agents create \
  --name "Example CLI Agent" \
  --instructions "You are a helpful assistant that answers questions concisely." \
  --json)

AGENT_ID=$(echo "$AGENT_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "  Created agent: $AGENT_ID"
echo ""

# 2. Get agent details
echo "→ Agent details:"
aix agents get "$AGENT_ID"
echo ""

# 3. Run the agent
echo "→ Running agent..."
aix agents run "$AGENT_ID" --query "What are the three laws of robotics?" --timeout 120
echo ""

# 4. Add a tool to the agent (replace TOOL_ID with a real one from `aix tools list`)
# echo "→ Adding tool to agent..."
# aix agents update "$AGENT_ID" --add-tool <TOOL_ID>
# aix agents get "$AGENT_ID"

# 5. Create an agent with tools attached from the start
# echo "→ Creating agent with tools..."
# aix agents create \
#   --name "Research Agent" \
#   --instructions "You research topics using your tools." \
#   --tool <TOOL_ID_1> \
#   --tool <TOOL_ID_2>

# 6. Create a multi-agent setup with sub-agents
# echo "→ Creating orchestrator agent..."
# aix agents create \
#   --name "Orchestrator" \
#   --instructions "You coordinate sub-agents to solve complex tasks." \
#   --subagent <AGENT_ID_1> \
#   --subagent <AGENT_ID_2>

# 7. Clean up
echo "→ Deleting agent..."
aix agents delete "$AGENT_ID" --force
echo ""

echo "Done! Agent lifecycle complete."
echo ""
echo "Tip: Use --tool and --subagent flags to attach capabilities:"
echo "  aix agents create --name 'My Agent' --tool <ID> --tool <ID>"
echo "  aix agents update <AGENT_ID> --add-tool <TOOL_ID>"
echo "  aix agents create --name 'Orchestrator' --subagent <AGENT_ID>"
