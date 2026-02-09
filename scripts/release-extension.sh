#!/usr/bin/env bash

set -euo pipefail

MANIFEST="extension/chrome/manifest.json"

# Get current version from manifest
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
cd extension/chrome && zip -r ../../laterfeed-chrome-extension.zip .

echo "Created laterfeed-chrome-extension.zip"
