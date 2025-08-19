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

# Find the header line for the version (supports optional date after dash)
START_LINE=$(grep -n -E "^## \\[$VERSION\\]( - .*)?$" CHANGELOG.md | head -n1 | cut -d: -f1 || true)
if [[ -z "${START_LINE:-}" ]]; then
  echo "Error: could not find section header for version $VERSION in CHANGELOG.md" >&2
  exit 1
fi

# Find the next header start line after START_LINE (or set to EOF)
END_LINE=$(awk -v s="$START_LINE" 'NR>s && /^## \[/ {print NR; exit}' CHANGELOG.md)
if [[ -z "${END_LINE:-}" ]]; then
  END_LINE=$(wc -l < CHANGELOG.md)
else
  END_LINE=$((END_LINE - 1))
fi

# Extract everything between the header and the next header
sed -n "$((START_LINE + 1)),$END_LINE p" CHANGELOG.md > "$NOTES_FILE"

if [[ ! -s "$NOTES_FILE" ]]; then
  echo "Error: found header for $VERSION but no content under it in CHANGELOG.md" >&2
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
