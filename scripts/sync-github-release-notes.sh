#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?release tag is required}"
NOTES_FILE="${RUNNER_TEMP:-/tmp}/github-release-notes-${TAG}.md"
TARGET_COMMITISH="${2:-}"

if ! command -v gh >/dev/null 2>&1; then
  echo "::error::GitHub CLI is required to generate release notes"
  exit 1
fi

if [ -z "${GITHUB_REPOSITORY:-}" ]; then
  echo "::error::GITHUB_REPOSITORY is required to generate release notes"
  exit 1
fi

if [ -z "${TARGET_COMMITISH}" ]; then
  TARGET_COMMITISH="$(git rev-parse HEAD)"
fi

if ! gh release view "${TAG}" --repo "${GITHUB_REPOSITORY}" >/dev/null 2>&1; then
  echo "::error::GitHub Release ${TAG} does not exist. Publish the release first, then run this workflow."
  exit 1
fi

BODY="$(gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/${GITHUB_REPOSITORY}/releases/generate-notes" \
  -f "tag_name=${TAG}" \
  -f "target_commitish=${TARGET_COMMITISH}" \
  --jq '.body // ""')"

if [ -z "${BODY}" ]; then
  echo "::error::GitHub generated an empty release note body for ${TAG}"
  exit 1
fi

printf '%s\n' "${BODY}" > "${NOTES_FILE}"
gh release edit "${TAG}" --repo "${GITHUB_REPOSITORY}" --notes-file "${NOTES_FILE}"
echo "GitHub generated release notes synced for ${TAG}."
