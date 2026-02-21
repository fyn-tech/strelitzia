#!/bin/bash
#
# Runs in the CONTAINER via devcontainer postStartCommand.
# Ensures git identity, safe directory, and SSH host aliases are configured.
#

IDENTITY_FILE=".devcontainer/.git-identity"

git config --global --add safe.directory "$(pwd)"

# --- Identity ---

if git config user.name > /dev/null 2>&1 && git config user.email > /dev/null 2>&1; then
    echo "Git identity: $(git config user.name) <$(git config user.email)>"
    rm -f "$IDENTITY_FILE"
else
    if [ -f "$IDENTITY_FILE" ]; then
        NAME=$(git config --file "$IDENTITY_FILE" user.name 2>/dev/null)
        EMAIL=$(git config --file "$IDENTITY_FILE" user.email 2>/dev/null)

        if [ -n "$NAME" ] && [ -n "$EMAIL" ]; then
            git config --global user.name "$NAME"
            git config --global user.email "$EMAIL"
            echo "Git identity set from host: $NAME <$EMAIL>"
            rm -f "$IDENTITY_FILE"
        fi
    fi

    if ! git config user.name > /dev/null 2>&1; then
        echo ""
        echo "WARNING: Git user.name and user.email are not configured."
        echo "Please run:"
        echo "  git config --global user.name 'Your Name'"
        echo "  git config --global user.email 'your@email.com'"
        echo ""
    fi
fi

# --- SSH host alias rewriting ---
# Rewrites unresolvable github.com-<alias> SSH hosts to github.com.
# This handles the common multi-identity SSH pattern where ~/.ssh/config
# maps aliases like github.com-work to github.com. That config isn't
# available inside the container, but the SSH agent (with the right keys) is.

for REMOTE_URL in $(git remote -v 2>/dev/null | awk '{print $2}' | sort -u); do
    HOST=$(echo "$REMOTE_URL" | sed -n 's/^git@\([^:]*\):.*/\1/p')
    [ -z "$HOST" ] && continue
    [ "$HOST" = "github.com" ] && continue

    if echo "$HOST" | grep -q '^github\.com-'; then
        if ! ssh -o ConnectTimeout=2 -o StrictHostKeyChecking=no "$HOST" true 2>/dev/null; then
            git config --global url."git@github.com:".insteadOf "git@${HOST}:"
            echo "SSH rewrite: git@${HOST}: -> git@github.com:"
        fi
    fi
done
