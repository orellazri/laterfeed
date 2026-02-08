#!/usr/bin/env bash

set -euo pipefail

# Get current version from latest git tag
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "No tags found")
echo "Current version: $CURRENT_VERSION"

# Prompt for the new version tag
echo "Enter the new version tag (without 'v' prefix, e.g. 1.0.1):"
read VERSION

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z (e.g. 1.0.1)"
    exit 1
fi

# Check if tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "Tag v$VERSION already exists. Deleting and recreating..."
    git tag -d "v$VERSION"
    git push origin ":refs/tags/v$VERSION"
fi

# Create and push the tag
git tag "v$VERSION"
git push origin main
git push --tags

echo "✅ Version $VERSION pushed!"
