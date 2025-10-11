#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
#
# SPDX-License-Identifier: MIT

set -euo pipefail

show_usage() {
  cat <<'USAGE'
Usage: prepare-metrics-image.sh [--token TOKEN]

Pull and tag lowlighter/metrics prebuilt image for GitHub Actions use.
The script pulls the latest released image from ghcr.io and re-tags it
with the exact name expected by the lowlighter/metrics action.
USAGE
}

TOKEN=""
while (($# > 0)); do
  case "$1" in
    --token)
      if (($# < 2)); then
        echo "--token requires a value" >&2
        exit 1
      fi
      TOKEN="$2"
      shift 2
      ;;
    --help|-h)
      show_usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      show_usage >&2
      exit 1
      ;;
  esac
done

GHCR_HOST="ghcr.io"
REPO_SLUG="lowlighter/metrics"
API_URL="https://api.github.com/repos/${REPO_SLUG}/releases/latest"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required to discover the latest metrics release" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to parse the GitHub API response" >&2
  exit 1
fi

RELEASE_JSON="$(curl -fsSL "${API_URL}")"
RELEASE_TAG="$(printf '%s' "${RELEASE_JSON}" | jq -r '.tag_name')"
if [ -z "${RELEASE_TAG}" ] || [ "${RELEASE_TAG}" = "null" ]; then
  echo "Unable to determine the latest metrics release tag" >&2
  exit 1
fi

METRICS_VERSION="${RELEASE_TAG#v}"
IFS='.' read -r MAJOR MINOR PATCH <<<"${METRICS_VERSION}"
if [ -z "${MAJOR}" ] || [ -z "${MINOR}" ]; then
  echo "Unexpected metrics version format: ${METRICS_VERSION}" >&2
  exit 1
fi

PREBUILT_TAG="v${MAJOR}.${MINOR}"
REMOTE_IMAGE="${GHCR_HOST}/${REPO_SLUG}:${PREBUILT_TAG}"

# lowlighter/metrics action expects this exact tag format
LOCAL_IMAGE="metrics:forked-${METRICS_VERSION}"

echo "Preparing metrics docker image ${LOCAL_IMAGE} from ${REMOTE_IMAGE}"

# Remove existing image to prevent stale data
if docker image inspect "${LOCAL_IMAGE}" >/dev/null 2>&1; then
  echo "Removing cached image ${LOCAL_IMAGE}"
  docker rmi "${LOCAL_IMAGE}" >/dev/null 2>&1 || true
fi

# Authenticate to ghcr.io if token provided
if [ -n "${TOKEN}" ]; then
  if printf '%s' "${TOKEN}" | docker login "${GHCR_HOST}" -u "${GITHUB_ACTOR:-github-actions[bot]}" --password-stdin >/dev/null 2>&1; then
    echo "Authenticated to ${GHCR_HOST}"
  else
    echo "Warning: unable to authenticate to ${GHCR_HOST}; attempting anonymous pull" >&2
  fi
fi

echo "Pulling prebuilt image ${REMOTE_IMAGE}"
if ! docker pull "${REMOTE_IMAGE}" >/dev/null 2>&1; then
  echo "Failed to pull ${REMOTE_IMAGE}. The action will build from source." >&2
  exit 0
fi

echo "Tagging ${REMOTE_IMAGE} as ${LOCAL_IMAGE}"
docker tag "${REMOTE_IMAGE}" "${LOCAL_IMAGE}"

echo "Successfully prepared ${LOCAL_IMAGE}"
