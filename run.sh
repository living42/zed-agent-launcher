#!/usr/bin/env sh
set -e

REPO="living42/zed-agent-launcher"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/zed-agent-launcher"
VERSION="${ZED_AGENT_LAUNCHER_VERSION:-latest}"
UPDATE_TTL="${ZED_AGENT_LAUNCHER_UPDATE_TTL:-86400}" # Default: 24 hours (86400 seconds)
FORCE_UPDATE=0

# Self-cache run.sh if executed via pipe or external location
SELF_SCRIPT="${CACHE_DIR}/run.sh"
mkdir -p "$CACHE_DIR"
if [ "$0" != "$SELF_SCRIPT" ] && [ -f "$0" ]; then
  cp "$0" "$SELF_SCRIPT" 2>/dev/null || true
  chmod +x "$SELF_SCRIPT" 2>/dev/null || true
fi

# Parse flags
while [ $# -gt 0 ]; do
  case "$1" in
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    --version=*)
      VERSION="${1#*=}"
      shift
      ;;
    -u|--update)
      FORCE_UPDATE=1
      shift
      ;;
    -h|--help-wrapper)
      echo "zed-agent-launcher auto-download & auto-update wrapper"
      echo ""
      echo "Usage: run.sh [options] [-- [zed-agent-launcher args]]"
      echo ""
      echo "Options:"
      echo "  -v, --version <TAG>   Pin specific version to download/run (default: latest)"
      echo "  -u, --update          Force check for latest version updates"
      echo "  -h, --help-wrapper    Show this wrapper help message"
      echo ""
      echo "Environment Variables:"
      echo "  ZED_AGENT_LAUNCHER_VERSION    Pin version tag (default: latest)"
      echo "  ZED_AGENT_LAUNCHER_UPDATE_TTL Update check frequency in seconds (default: 86400)"
      exit 0
      ;;
    --)
      shift
      break
      ;;
    *)
      break
      ;;
  esac
done

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Darwin) OS_TARGET="apple-darwin" ;;
  Linux)  OS_TARGET="unknown-linux-gnu" ;;
  *)
    echo "Error: Unsupported operating system: $OS" >&2
    exit 1
    ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH_TARGET="x86_64" ;;
  arm64|aarch64) ARCH_TARGET="aarch64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
BINARY_NAME="zed-agent-launcher-${TARGET}"

# Helper function to fetch URL
fetch_url() {
  url="$1"
  out="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$out"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$out" "$url"
  else
    return 1
  fi
}

CHECK_FILE="${CACHE_DIR}/.last_update_check"
CURRENT_VERSION_FILE="${CACHE_DIR}/.current_version"
NEED_DOWNLOAD=0

if [ "$VERSION" != "latest" ]; then
  # Pinned version requested
  TARGET_VERSION="$VERSION"
  BINARY_PATH="${CACHE_DIR}/${TARGET_VERSION}/${BINARY_NAME}"
  if [ ! -f "$BINARY_PATH" ]; then
    NEED_DOWNLOAD=1
  fi
else
  # Unpinned version (latest)
  CACHED_VERSION=""
  [ -f "$CURRENT_VERSION_FILE" ] && CACHED_VERSION="$(cat "$CURRENT_VERSION_FILE")"
  
  NOW=$(date +%s 2>/dev/null || echo 0)
  LAST_CHECK=0
  [ -f "$CHECK_FILE" ] && LAST_CHECK="$(cat "$CHECK_FILE")"
  
  TIME_DIFF=$((NOW - LAST_CHECK))
  
  # Check if cached binary exists
  if [ -n "$CACHED_VERSION" ] && [ -f "${CACHE_DIR}/${CACHED_VERSION}/${BINARY_NAME}" ]; then
    BINARY_PATH="${CACHE_DIR}/${CACHED_VERSION}/${BINARY_NAME}"
    TARGET_VERSION="$CACHED_VERSION"
    
    # Check if TTL expired or forced update
    if [ "$FORCE_UPDATE" -eq 1 ] || [ "$TIME_DIFF" -ge "$UPDATE_TTL" ] || [ "$NOW" -eq 0 ]; then
      echo "Checking for latest release update..." >&2
      LATEST_TAG=""
      TMP_TAG="${CACHE_DIR}/.tmp_tag_$$"
      if fetch_url "https://api.github.com/repos/${REPO}/releases/latest" "$TMP_TAG"; then
        LATEST_TAG=$(grep -o '"tag_name": *"[^"]*"' "$TMP_TAG" | head -n 1 | cut -d'"' -f4)
        rm -f "$TMP_TAG"
      fi
      
      if [ -n "$LATEST_TAG" ]; then
        echo "$NOW" > "$CHECK_FILE"
        if [ "$LATEST_TAG" != "$CACHED_VERSION" ]; then
          echo "New version available: ${LATEST_TAG} (current: ${CACHED_VERSION})" >&2
          TARGET_VERSION="$LATEST_TAG"
          BINARY_PATH="${CACHE_DIR}/${TARGET_VERSION}/${BINARY_NAME}"
          if [ ! -f "$BINARY_PATH" ]; then
            NEED_DOWNLOAD=1
          fi
        fi
      else
        echo "Note: Could not check for updates (offline or rate limited). Using cached ${CACHED_VERSION}." >&2
      fi
    fi
  else
    # No cached binary exists, must fetch latest tag and download
    NEED_DOWNLOAD=1
    TMP_TAG="${CACHE_DIR}/.tmp_tag_$$"
    LATEST_TAG=""
    if fetch_url "https://api.github.com/repos/${REPO}/releases/latest" "$TMP_TAG"; then
      LATEST_TAG=$(grep -o '"tag_name": *"[^"]*"' "$TMP_TAG" | head -n 1 | cut -d'"' -f4)
      rm -f "$TMP_TAG"
    fi
    TARGET_VERSION="${LATEST_TAG:-latest}"
    BINARY_PATH="${CACHE_DIR}/${TARGET_VERSION}/${BINARY_NAME}"
  fi
fi

# Download binary if needed
if [ "$NEED_DOWNLOAD" -eq 1 ] || [ ! -f "$BINARY_PATH" ]; then
  VERSION_DIR="$(dirname "$BINARY_PATH")"
  mkdir -p "$VERSION_DIR"
  
  if [ "$TARGET_VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TARGET_VERSION}/${BINARY_NAME}"
  fi

  echo "Downloading zed-agent-launcher (${TARGET_VERSION}) for ${TARGET}..." >&2
  TMP_FILE="${BINARY_PATH}.tmp.$$"
  
  if fetch_url "$DOWNLOAD_URL" "$TMP_FILE"; then
    chmod +x "$TMP_FILE"
    mv "$TMP_FILE" "$BINARY_PATH"
    if [ "$VERSION" = "latest" ] && [ "$TARGET_VERSION" != "latest" ]; then
      echo "$TARGET_VERSION" > "$CURRENT_VERSION_FILE"
      echo "$(date +%s 2>/dev/null || echo 0)" > "$CHECK_FILE"
    fi
    echo "Cached zed-agent-launcher at ${BINARY_PATH}" >&2
  else
    rm -f "$TMP_FILE"
    if [ -n "$CACHED_VERSION" ] && [ -f "${CACHE_DIR}/${CACHED_VERSION}/${BINARY_NAME}" ]; then
      echo "Download failed. Falling back to cached version ${CACHED_VERSION}." >&2
      BINARY_PATH="${CACHE_DIR}/${CACHED_VERSION}/${BINARY_NAME}"
    else
      echo "Error: Failed to download zed-agent-launcher binary." >&2
      exit 1
    fi
  fi
fi

# Reattach stdin to /dev/tty if stdin is piped (e.g. curl ... | sh)
if [ ! -t 0 ] && [ -c /dev/tty ]; then
  exec "$BINARY_PATH" "$@" < /dev/tty
else
  exec "$BINARY_PATH" "$@"
fi
