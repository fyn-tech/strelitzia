#!/bin/bash
#
# Runs on the HOST via devcontainer initializeCommand.
# Captures the resolved git identity and writes it to a file
# that the container-side setup script can read.
#

IDENTITY_FILE=".devcontainer/.git-identity"

NAME=$(git config user.name 2>/dev/null)
EMAIL=$(git config user.email 2>/dev/null)

if [ -n "$NAME" ] && [ -n "$EMAIL" ]; then
    git config --file "$IDENTITY_FILE" user.name "$NAME"
    git config --file "$IDENTITY_FILE" user.email "$EMAIL"
fi
