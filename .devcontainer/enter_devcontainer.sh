#!/bin/bash

# Extract the container name from the --name runtime argument in devcontainer.json
CONTAINER_NAME=$(jq -r '.runArgs | (index("--name") + 1) as $i | .[$i]' "$(dirname "$0")/../.devcontainer/devcontainer.json")

# If the container is running, open a bash shell in the container in interactive mode.
if [ -n "$CONTAINER_NAME" ]; then
  docker exec -it "$CONTAINER_NAME" bash
  exit 0
else
  echo "No devcontainer found."
fi