#!/usr/bin/env bash
set -eu

#  If the version is not specified, use the latest version.
VERSION=${HOP_VERSION:-"latest"}

# If the install directory is not set, set it to a default value
INSTALL_DIR=${HOP_INSTALL_DIR:-"/usr/local/bin"}

PLATFORM=`uname -s`
ARCH=`uname -m`

if [[ $PLATFORM == CYGWIN* ]] || [[ $PLATFORM == MINGW* ]] || [[ $PLATFORM == MSYS* ]]; then
    PLATFORM="Windows"
fi

if [[ $ARCH == armv8* ]] || [[ $ARCH == arm64* ]] || [[ $ARCH == aarch64* ]]; then
    ARCH="aarch64"
fi

if [[ $ARCH == armv6* ]] || [[ $ARCH == armv7* ]]; then
    ARCH="armv7"
fi

if [ -z ${INSTALL_PATH+x} ]; then
    INSTALL_PATH="${INSTALL_DIR}/hop"
fi

DOWNLOAD_URL="https://github.com/hopinc/hop_cli/releases/download/${VERSION}/hop-${PLATFORM}_${ARCH}.$([ $PLATFORM = "Windows" ] && echo "zip" || echo "tar.gz"  )"

echo "This script will automatically install hop (${VERSION}) for you."
echo "Installation path: ${INSTALL_PATH}"

if [ "x$(id -u)" == "x0" ]; then
    echo "Warning: this script is currently running as root. This is dangerous. "
    echo "         Instead run it as normal user. We will sudo as needed."
fi

if [ -f "$INSTALL_PATH" ]; then
    echo "error: hop is already installed."
    echo "  run \"hop update\" to update to latest version"
    exit 1
fi

if ! hash curl 2> /dev/null; then
    echo "error: you do not have 'curl' installed which is required for this script."
    exit 1
fi

TEMP_FILE=`mktemp "${TMPDIR:-/tmp}/.hop"`
TEMP_HEADER_FILE=`mktemp "${TMPDIR:-/tmp}/.hop-headers"`

cleanup() {
    rm -f "$TEMP_FILE"
    rm -f "$TEMP_HEADER_FILE"
}

trap cleanup EXIT
HTTP_CODE=$(curl -SL --progress-bar "$DOWNLOAD_URL" -D "$TEMP_HEADER_FILE" --output "$TEMP_FILE" --write-out "%{http_code}")
if [[ ${HTTP_CODE} -lt 200 || ${HTTP_CODE} -gt 299 ]]; then
    echo "error: your platform and architecture (${ARCH}-${PLATFORM}) is unsupported."
    exit 1
fi

chmod 0755 "$TEMP_FILE"
if ! mv "$TEMP_FILE" "$INSTALL_PATH" 2> /dev/null; then
    sudo -k mv "$TEMP_FILE" "$INSTALL_PATH"
fi

echo "Installed $("$INSTALL_PATH" --version)"

echo 'Done!'