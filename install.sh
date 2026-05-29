#!/usr/bin/env sh
set -eu

# Override when needed:
#   RFDISK_REPO=another-user/rfdisk sh install.sh
REPO="${RFDISK_REPO:-EasonLin-X/rfdisk}"
VERSION="${RFDISK_VERSION:-latest}"
INSTALL_DIR="${RFDISK_INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="rfdisk"

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command not found: $1" >&2
        exit 1
    fi
}

download() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        echo "error: curl or wget is required" >&2
        exit 1
    fi
}

install_file() {
    src="$1"
    dst="$2"

    if [ "$(id -u)" -eq 0 ]; then
        mkdir -p "$(dirname "$dst")"
        cp "$src" "$dst"
        chmod 0755 "$dst"
    else
        need_cmd sudo
        sudo mkdir -p "$(dirname "$dst")"
        sudo cp "$src" "$dst"
        sudo chmod 0755 "$dst"
    fi
}

os="$(uname -s)"
arch="$(uname -m)"

if [ "$os" != "Linux" ]; then
    echo "error: rfdisk only supports Linux" >&2
    exit 1
fi

case "$arch" in
    x86_64|amd64)
        asset="rfdisk-linux-x86_64.tar.gz"
        ;;
    aarch64|arm64)
        asset="rfdisk-linux-aarch64.tar.gz"
        ;;
    *)
        echo "error: unsupported architecture: $arch" >&2
        echo "supported: x86_64, aarch64" >&2
        exit 1
        ;;
esac

need_cmd uname
need_cmd tar
need_cmd mktemp

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/$asset"
else
    url="https://github.com/$REPO/releases/download/$VERSION/$asset"
fi

echo "Installing rfdisk"
echo "Repository: $REPO"
echo "Version:    $VERSION"
echo "Asset:      $asset"
echo "Target:     $INSTALL_DIR/$BINARY_NAME"
echo
echo "Downloading: $url"

download "$url" "$tmp_dir/$asset"
tar -xzf "$tmp_dir/$asset" -C "$tmp_dir"

if [ ! -f "$tmp_dir/$BINARY_NAME" ]; then
    echo "error: release archive does not contain $BINARY_NAME" >&2
    exit 1
fi

install_file "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo
echo "rfdisk installed successfully."
echo "Run with:"
echo "  sudo $BINARY_NAME"
