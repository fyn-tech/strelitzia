#!/bin/bash
#
# Runs in the CONTAINER via devcontainer postStartCommand.
# Ensures git identity and safe directory are configured.
#

IDENTITY_FILE=".devcontainer/.git-identity"

git config --global --add safe.directory "$(pwd)"

if git config user.name > /dev/null 2>&1 && git config user.email > /dev/null 2>&1; then
    echo "Git identity: $(git config user.name) <$(git config user.email)>"
    rm -f "$IDENTITY_FILE"
    exit 0
fi

if [ -f "$IDENTITY_FILE" ]; then
    NAME=$(git config --file "$IDENTITY_FILE" user.name 2>/dev/null)
    EMAIL=$(git config --file "$IDENTITY_FILE" user.email 2>/dev/null)

    if [ -n "$NAME" ] && [ -n "$EMAIL" ]; then
        git config --global user.name "$NAME"
        git config --global user.email "$EMAIL"
        echo "Git identity set from host: $NAME <$EMAIL>"
        rm -f "$IDENTITY_FILE"
        exit 0
    fi
fi

echo ""
echo "WARNING: Git user.name and user.email are not configured."
echo "Please run:"
echo "  git config --global user.name 'Your Name'"
echo "  git config --global user.email 'your@email.com'"
echo ""
