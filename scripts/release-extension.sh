#!/usr/bin/env bash

set -euo pipefail

BROWSER="${1:-}"

if [[ -z "$BROWSER" ]]; then
	echo "Usage: release-extension.sh <chrome|firefox>"
	exit 1
fi

if [[ "$BROWSER" != "chrome" && "$BROWSER" != "firefox" ]]; then
	echo "Error: Browser must be one of: chrome, firefox"
	exit 1
fi

# Get current version from the manifest
MANIFEST="extension/${BROWSER}/manifest.json"
CURRENT_VERSION=$(grep -o '"version": "[^"]*"' "$MANIFEST" | head -1 | cut -d'"' -f4)
echo "Current extension version: $CURRENT_VERSION"

# Prompt for the new version
echo "Enter the new version (e.g. 1.0.1):"
read VERSION

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "Error: Version must be in format X.Y.Z (e.g. 1.0.1)"
	exit 1
fi

# Update version in manifest.json
if [[ "$(uname)" == "Darwin" ]]; then
	sed -i '' "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$VERSION\"/" "$MANIFEST"
else
	sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$VERSION\"/" "$MANIFEST"
fi

echo "Updated $MANIFEST to version $VERSION"

# Create zip
mkdir -p dist
(cd "extension/$BROWSER" && zip -r "../../dist/laterfeed-${BROWSER}-extension.zip" .)

echo "Created dist/laterfeed-${BROWSER}-extension.zip"
