#!/usr/bin/env bash
set -euo pipefail

# release_from_changelog.sh
# Create a GitHub release using notes extracted from CHANGELOG.md for a given version.
#
# Usage:
#   scripts/release_from_changelog.sh <version> [--latest]
#
# Examples:
#   scripts/release_from_changelog.sh 0.1.0 --latest
#   scripts/release_from_changelog.sh 0.1.1
#
# Requirements:
#   - GitHub CLI (gh) installed and authenticated: `gh auth login`
#   - CHANGELOG.md following Keep a Changelog with headings like `## [0.1.0] - YYYY-MM-DD`

if [[ ${#} -lt 1 ]]; then
  echo "Usage: $0 <version> [--latest]" >&2
  exit 1
fi

VERSION="$1"
shift || true

MARK_LATEST=false
if [[ ${#} -gt 0 ]]; then
  if [[ "$1" == "--latest" ]]; then
    MARK_LATEST=true
    shift || true
  fi
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: gh (GitHub CLI) is not installed. See https://cli.github.com" >&2
  exit 1
fi

if [[ ! -f CHANGELOG.md ]]; then
  echo "Error: CHANGELOG.md not found in current directory" >&2
  exit 1
fi

# Extract notes for the version between '## [VERSION]' and the next '## ['
# Preserve markdown formatting.
NOTES_FILE=$(mktemp)
trap 'rm -f "$NOTES_FILE"' EXIT

awk -v ver="$VERSION" '
  BEGIN { in_section=0 }
  /^## \[/ {
    if (in_section) exit 0
    if ($0 ~ "^## \[" ver "\]") { in_section=1; next }
  }
  { if (in_section) print }
' CHANGELOG.md > "$NOTES_FILE"

if [[ ! -s "$NOTES_FILE" ]]; then
  echo "Error: could not find section for version $VERSION in CHANGELOG.md" >&2
  exit 1
fi

# Create the release. Uses existing tag v<version>.
ARGS=("v$VERSION" "--title" "v$VERSION" "--notes-file" "$NOTES_FILE")
if [[ "$MARK_LATEST" == true ]]; then
  ARGS+=("--latest")
fi

echo "Creating GitHub release v$VERSION using notes from CHANGELOG.md ..."
set -x
gh release create "${ARGS[@]}"
