#!/usr/bin/env bash
set -euo pipefail

# SIAR Wiki Deployment Script
# Pushes the contents of sys-arch/ to the GitHub Wiki Git repository.
# Prerequisite: Enable Wiki on GitHub (Repositories -> Settings -> Features -> Wikis -> Create first page)

WIKI_REMOTE="https://github.com/irshadali5/siar.wiki.git"
TEMP_DIR="$(mktemp -d)"

echo "==> Preparing SIAR Architecture Wiki deployment..."
trap 'rm -rf "${TEMP_DIR}"' EXIT

cd "$(dirname "$0")/.."
cp -r sys-arch/* "${TEMP_DIR}/"

cd "${TEMP_DIR}"
git init
git branch -M master
git add .
git commit -m "docs(wiki): sync 60 comprehensive SIAR system architecture specifications"

echo "==> Deploying to GitHub Wiki (${WIKI_REMOTE})..."
git push --force "${WIKI_REMOTE}" master || {
    echo "[-] Push failed. Please make sure the Wiki is initialized on GitHub:"
    echo "    1. Navigate to https://github.com/irshadali5/siar/wiki"
    echo "    2. Click 'Create the first page' and save."
    echo "    3. Re-run this script."
    exit 1
}

echo "[+] Successfully deployed SIAR Architecture Wiki to GitHub!"
