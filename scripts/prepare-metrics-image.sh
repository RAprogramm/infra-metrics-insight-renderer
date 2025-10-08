#!/usr/bin/env bash
set -euo pipefail

show_usage() {
  cat <<'USAGE'
Usage: prepare-metrics-image.sh [--token TOKEN]

Ensure a usable lowlighter/metrics docker image is available locally.
The script attempts to pull the latest released image from GHCR and
re-tags it so GitHub actions can reuse it without rebuilding. When the
pull fails, it clones the matching release and builds a patched image
that installs the additional xz-utils dependency required by Nokogiri.
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
IFS='.' read -r MAJOR MINOR _ <<<"${METRICS_VERSION}"
if [ -z "${MAJOR}" ] || [ -z "${MINOR}" ]; then
  echo "Unexpected metrics version format: ${METRICS_VERSION}" >&2
  exit 1
fi
PREBUILT_TAG="v${MAJOR}.${MINOR}"
LOCAL_IMAGE="metrics:${METRICS_VERSION}"
REMOTE_IMAGE="${GHCR_HOST}/${REPO_SLUG}:${PREBUILT_TAG}"

echo "Preparing metrics docker image ${LOCAL_IMAGE} (prebuilt tag ${PREBUILT_TAG})"

if docker image inspect "${LOCAL_IMAGE}" >/dev/null 2>&1; then
  echo "Removing cached image ${LOCAL_IMAGE} to prevent stale data"
  docker rmi "${LOCAL_IMAGE}" >/dev/null 2>&1 || true
fi

if [ -n "${TOKEN}" ]; then
  if ! printf '%s' "${TOKEN}" | docker login "${GHCR_HOST}" -u "${GITHUB_ACTOR:-github-actions[bot]}" --password-stdin >/dev/null 2>&1; then
    echo "Warning: unable to authenticate to ${GHCR_HOST}; attempting anonymous pull" >&2
  else
    echo "Authenticated to ${GHCR_HOST} for metrics image pull"
  fi
fi

echo "Attempting to pull prebuilt image ${REMOTE_IMAGE}" >&2
if docker pull "${REMOTE_IMAGE}" >/dev/null 2>&1; then
  echo "Pulled ${REMOTE_IMAGE}; tagging as ${LOCAL_IMAGE}" >&2
  docker tag "${REMOTE_IMAGE}" "${LOCAL_IMAGE}"
  exit 0
fi

echo "Falling back to building metrics image ${LOCAL_IMAGE} locally" >&2

if ! command -v git >/dev/null 2>&1; then
  echo "git is required to clone ${REPO_SLUG}" >&2
  exit 1
fi

TEMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

git clone --depth 1 --branch "${RELEASE_TAG}" "https://github.com/${REPO_SLUG}.git" "${TEMP_DIR}" >/dev/null 2>&1

DOCKERFILE_PATH="${TEMP_DIR}/Dockerfile"
if [ ! -f "${DOCKERFILE_PATH}" ]; then
  echo "Dockerfile not found in cloned repository" >&2
  exit 1
fi

if ! sed -i 's/apt-get install -y ruby-full git g++ cmake pkg-config libssl-dev/apt-get install -y ruby-full git g++ cmake pkg-config libssl-dev xz-utils/' "${DOCKERFILE_PATH}"; then
  echo "Failed to patch Dockerfile with xz-utils dependency" >&2
  exit 1
fi

docker build -t "${LOCAL_IMAGE}" "${TEMP_DIR}"
