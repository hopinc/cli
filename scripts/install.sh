#!/usr/bin/env sh
set -eu

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

if [[ $ARCH == armv6* ]] || [[ $ARCH == armv7* ]] || [[ $ARCH == arm7* ]]; then
    ARCH="arm"
fi

if [ -z ${INSTALL_PATH+x} ]; then
    INSTALL_PATH="${INSTALL_DIR}/hop"
fi

if [[ -z "${HOP_VERSION:-""}" ]]; then
    VERSION="latest"
    DOWNLOAD_URL="https://github.com/hopinc/hop_cli/releases/latest/download/hop-${ARCH}-${PLATFORM}.$([ $PLATFORM = "Windows" ] && echo "zip" || echo "tar.gz"  )"
else
    VERSION="${HOP_VERSION}"
    DOWNLOAD_URL="https://github.com/hopinc/hop_cli/releases/download/${VERSION}/hop-${ARCH}-${PLATFORM}.$([ $PLATFORM = "Windows" ] && echo "zip" || echo "tar.gz"  )"
fi

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

if ! command -v curl 2> /dev/null; then
    echo "error: you do not have 'curl' installed which is required for this script."
    exit 1
fi

TEMP_FILE=`mktemp "${TMPDIR:-/tmp}/hop-tar.XXXXXXXXXX.tar.gz"`
TEMP_HEADER_FILE=`mktemp "${TMPDIR:-/tmp}/hop-headers.XXXXXXXXXX"`

cleanup() {
    rm -f "$TEMP_FILE"
    rm -f "$TEMP_HEADER_FILE"
}

echo "Downloading $DOWNLOAD_URL"

trap cleanup EXIT
HTTP_CODE=$(curl -SL --progress-bar "$DOWNLOAD_URL" -D "$TEMP_HEADER_FILE" --output "$TEMP_FILE" --write-out "%{http_code}")
if [[ ${HTTP_CODE} -lt 200 || ${HTTP_CODE} -gt 299 ]]; then
    echo "error: your platform and architecture (${ARCH}-${PLATFORM}) is unsupported."
    exit 1
fi

EXTRACT_DIR=`mktemp -d "${TMPDIR:-/tmp}/hop-extract-dir.XXXXXXXXXX"`
EXTRACTED_FILE=`mktemp "${TMPDIR:-/tmp}/hop.XXXXXXXXXX"`

# untar or unzip the file
if [ $PLATFORM = "Windows" ]; then
    unzip -o "${TEMP_FILE}" -d "${EXTRACT_DIR}"
    mv "${EXTRACT_DIR}/hop.exe" "${EXTRACTED_FILE}"
else
    tar -xzf "${TEMP_FILE}" -C "${EXTRACT_DIR}"
    mv "${EXTRACT_DIR}/hop" "${EXTRACTED_FILE}"
fi

chmod 0755 "${EXTRACTED_FILE}"
if ! mv "${EXTRACTED_FILE}" "${INSTALL_PATH}" 2> /dev/null; then
    echo "sudo is required to install hop to ${INSTALL_DIR}"
    sudo -k mv "${EXTRACTED_FILE}" "${INSTALL_PATH}"
fi

# check if the current version supports completions
set +e
$INSTALL_PATH completions --help &> /dev/null
EXIT_CODE=$?
set -e

# omit on CI, Windows and on unsupported versions
if [[ -z "${CI:-""}" ]] && [ $PLATFORM != "Windows" ] && [  $EXIT_CODE -eq 0 ]; then
    # checks if any of the supported shells exists and if so, adds the hop completions to it
    
    if command -v fish 2> /dev/null; then
        echo "Installing fish completion"

        CMD="$INSTALL_PATH completions fish > /usr/share/fish/completions/hop.fish 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi
    
    if command -v zsh 2> /dev/null; then
        echo "Installing zsh completion"

        CMD="$INSTALL_PATH completions zsh > /usr/share/zsh/site-functions/_hop 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi

    if command -v bash 2> /dev/null; then
        echo "Installing bash completion"

        CMD="$INSTALL_PATH completions bash > /usr/share/bash-completion/completions/hop 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi
fi


echo "
             \\\\
        ,-~~~-\\\\_
       (        .\    Hop CLI is now installed
\\    / @\___(__--'    Start off by using: hop auth login
 \\  /
 hop!"