#!/usr/bin/env bash
set -euo pipefail

BROWSER="${BROWSER:-chromium}"

TARGET_USER="${_REMOTE_USER:-vscode}"
TARGET_HOME="${_REMOTE_USER_HOME:-/home/${TARGET_USER}}"

if [[ "$(id -u)" -ne 0 ]]; then
  echo "ERROR: this feature must be executed as root." >&2
  exit 1
fi

# Call binaries by absolute path because `su` on Debian/Ubuntu rebuilds PATH
# via pam_env and erases the Dockerfile-level containerEnv PATH.
NVM_CURRENT_BIN="/usr/local/share/nvm/current/bin"
NPX="${NVM_CURRENT_BIN}/npx"

if [[ ! -x "${NPX}" ]]; then
  echo "ERROR: npx not found at '${NPX}'." >&2
  echo "Make sure ghcr.io/devcontainers/features/node is installed before this feature." >&2
  exit 1
fi

# 1) System libraries via apt-get. playwright's install-deps shells out to apt,
#    so it must run as root.
"${NPX}" -y playwright@latest install-deps "${BROWSER}"

# 2) Browser binaries cached under the target user's ~/.cache/ms-playwright.
su "${TARGET_USER}" -c "HOME='${TARGET_HOME}' '${NPX}' -y playwright@latest install ${BROWSER}"

echo "playwright-mcp feature installed for user ${TARGET_USER}."
echo "The @playwright/mcp package itself is fetched on first invocation via npx."
