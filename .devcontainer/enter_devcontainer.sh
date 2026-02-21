#!/bin/bash

# Extract the container name from the --name runtime argument in devcontainer.json
CONTAINER_NAME=$(jq -r '.runArgs | (index("--name") + 1) as $i | .[$i] // empty' "$(dirname "$0")/devcontainer.json")

if [ -z "$CONTAINER_NAME" ]; then
  echo "Error: No --name found in devcontainer.json runArgs."
  exit 1
fi

# Check the container is actually running
if ! docker inspect --format='{{.State.Running}}' "$CONTAINER_NAME" 2>/dev/null | grep -q true; then
  echo "Error: Container '$CONTAINER_NAME' is not running."
  exit 1
fi

WORKSPACE_FOLDER="/workspaces/$(basename "$(cd "$(dirname "$0")/.." && pwd)")"

docker exec -it -e "TERM=${TERM:-xterm-256color}" -w "$WORKSPACE_FOLDER" "$CONTAINER_NAME" bash -l