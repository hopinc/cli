#!/usr/bin/env sh
set -eu

# SH compliant shell script to install Hop CLI
# This script is meant for quick & easy install.

if [ "x$(id -u)" = "x0" ]; then
    echo "Warning: this script is currently running as root. This is dangerous. "
    echo "         Instead run it as normal user. We will sudo as needed."
fi

if command -v hop 2> /dev/null; then
    echo "error: hop is already installed, please uninstall it first or"
    echo "  run \"hop update\" to update to latest version"
    exit 1
fi

# If the install directory is not set, set it to a default value
INSTALL_DIR=${HOP_INSTALL_DIR:-"/usr/local/bin"}
PRODUCT_NAME="hop"
ORG_NAME="hopinc"
REPO_NAME="cli"

PLATFORM=`uname -s`
ARCH=`uname -m`

case "${PLATFORM}" in
    CYGWIN*) PLATFORM="Windows" ;;
    MINGW*) PLATFORM="Windows" ;;
    MSYS*) PLATFORM="Windows" ;;
esac

case "${ARCH}" in
    armv8*) ARCH="aarch64" ;;
    arm64*) ARCH="aarch64" ;;
    aarch64*) ARCH="aarch64" ;;
    armv6*) ARCH="arm" ;;
    armv7*) ARCH="arm" ;;
    arm7*) ARCH="arm" ;;
esac

if [ -z ${INSTALL_PATH+x} ]; then
    INSTALL_PATH="${INSTALL_DIR}/${PRODUCT_NAME}"
fi

if [ -z "${HOP_VERSION:-""}" ]; then
    VERSION="latest"
    DOWNLOAD_URL="https://github.com/${ORG_NAME}/${REPO_NAME}/releases/latest/download/${PRODUCT_NAME}-${ARCH}-${PLATFORM}.$([ $PLATFORM = "Windows" ] && echo "zip" || echo "tar.gz"  )"
else
    VERSION="${HOP_VERSION}"
    DOWNLOAD_URL="https://github.com/${ORG_NAME}/${REPO_NAME}/releases/download/${VERSION}/${PRODUCT_NAME}-${ARCH}-${PLATFORM}.$([ $PLATFORM = "Windows" ] && echo "zip" || echo "tar.gz"  )"
fi

echo "This script will automatically install ${PRODUCT_NAME} (${VERSION}) for you."
echo "Installation path: ${INSTALL_PATH}"

if ! command -v curl 2> /dev/null; then
    echo "error: you do not have 'curl' installed which is required for this script."
    exit 1
fi

TEMP_FILE=`mktemp "${TMPDIR:-/tmp}/${PRODUCT_NAME}-tar.XXXXXXXXXX.tar.gz"`
TEMP_HEADER_FILE=`mktemp "${TMPDIR:-/tmp}/${PRODUCT_NAME}-headers.XXXXXXXXXX"`

cleanup() {
    rm -f "$TEMP_FILE"
    rm -f "$TEMP_HEADER_FILE"
}

echo "Downloading $DOWNLOAD_URL"

trap cleanup EXIT
HTTP_CODE=$(curl -SL --progress-bar "$DOWNLOAD_URL" -D "$TEMP_HEADER_FILE" --output "$TEMP_FILE" --write-out "%{http_code}")
if [ ${HTTP_CODE} -lt 200 ] || [ ${HTTP_CODE} -gt 299 ]; then
    echo "error: your platform and architecture (${ARCH}-${PLATFORM}) is unsupported."
    exit 1
fi

EXTRACT_DIR=`mktemp -d "${TMPDIR:-/tmp}/${PRODUCT_NAME}-extract-dir.XXXXXXXXXX"`
EXTRACTED_FILE=`mktemp "${TMPDIR:-/tmp}/${PRODUCT_NAME}.XXXXXXXXXX"`

# untar or unzip the file
if [ $PLATFORM = "Windows" ]; then
    unzip -o "${TEMP_FILE}" -d "${EXTRACT_DIR}"
    mv "${EXTRACT_DIR}/${PRODUCT_NAME}.exe" "${EXTRACTED_FILE}"
else
    tar -xzf "${TEMP_FILE}" -C "${EXTRACT_DIR}"
    mv "${EXTRACT_DIR}/${PRODUCT_NAME}" "${EXTRACTED_FILE}"
fi

chmod 0755 "${EXTRACTED_FILE}"
if ! mv "${EXTRACTED_FILE}" "${INSTALL_PATH}" 2> /dev/null; then
    echo "sudo is required to install ${PRODUCT_NAME} to ${INSTALL_DIR}"
    sudo -k mv "${EXTRACTED_FILE}" "${INSTALL_PATH}"
fi

# check if the current version supports completions
set +e
$INSTALL_PATH completions --help 2> /dev/null > /dev/null
EXIT_CODE=$?
set -e

# omit on CI, Windows and on unsupported versions
if [ -z "${CI:-""}" ] && [ $PLATFORM != "Windows" ] && [ $PLATFORM != "Darwin" ] && [  $EXIT_CODE -eq 0 ]; then
    # checks if any of the supported shells exists and if so, adds the hop completions to it
    
    if command -v fish 2> /dev/null; then
        echo "Installing fish completion"

        CMD="mkdir -p /usr/share/fish/completions"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null

        # redirect the possible warnings of update to /dev/null (shouldnt happen)
        CMD="$INSTALL_PATH completions fish > /usr/share/fish/completions/${PRODUCT_NAME}.fish 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi
    
    if command -v zsh 2> /dev/null; then
        echo "Installing zsh completion"

        CMD="mkdir -p /usr/share/zsh/site-functions"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null

        CMD="$INSTALL_PATH completions zsh > /usr/share/zsh/site-functions/_${PRODUCT_NAME} 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi

    if command -v bash 2> /dev/null; then
        echo "Installing bash completion"

        CMD="mkdir -p /usr/share/bash-completion/completions"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null

        CMD="$INSTALL_PATH completions bash > /usr/share/bash-completion/completions/${PRODUCT_NAME} 2> /dev/null"

        sh -c "$CMD" 2> /dev/null || sudo sh -c "$CMD" 2> /dev/null
    fi
fi

# confirm the installation user is the owner of the ~/.hop directory
if [ -d ~/.hop ]; then
    if [ "$(stat -c '%U' ~/.hop)" != "$(whoami)" ]; then
        echo "Changing ownership of ~/.hop to $(whoami)"
        sudo chown -R $(whoami):$(whoami) ~/.hop
    fi
fi


echo "
             \\\\
        ,-~~~-\\\\_
       (        .\    Hop CLI is now installed
\\    / @\___(__--'    Start off by using: hop auth login
 \\  /
 hop!"