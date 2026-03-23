#!/usr/bin/env bash
# lyxgc chktex quickstart - fetch prebuilt binary and run on sample.
# Usage: ./quickstart.sh
# Requires: curl, tar, internet
set -e
REPO="gmatht/lyxgc.rs"

# Detect OS and arch
OS=""
ARCH=""
case "$(uname -s)" in
  Linux*)   OS="linux";;
  Darwin*)  OS="macos";;
  *)        echo "Unsupported OS"; exit 1;;
esac
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64";;
  aarch64|arm64) ARCH="aarch64";;
  *)            echo "Unsupported arch: $ARCH"; exit 1;;
esac

# Resolve latest release
echo "Fetching latest release..."
RELEASE=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest")
TAG=$(echo "$RELEASE" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
if [ -z "$TAG" ]; then
  echo "No release found. Build from source: cd rs && cargo build --release"
  exit 1
fi

# Find asset URL
ASSET_NAME="chktex-${OS}-${ARCH}.tar.gz"
URL=$(echo "$RELEASE" | grep "browser_download_url" | grep "$ASSET_NAME" | head -1 | sed 's/.*"browser_download_url": *"\([^"]*\)".*/\1/')
if [ -z "$URL" ]; then
  echo "No $ASSET_NAME in release $TAG"
  exit 1
fi

# Download and extract
DIR=$(mktemp -d)
trap "rm -rf $DIR" EXIT
echo "Downloading $ASSET_NAME..."
curl -fsSL -o "$DIR/bin.tar.gz" "$URL"
tar xzf "$DIR/bin.tar.gz" -C "$DIR"
CHKTEX="$DIR/chktex-${OS}-${ARCH}"
[ -f "$CHKTEX" ] || CHKTEX=$(find "$DIR" -name chktex -type f | head -1)
if [ -z "$CHKTEX" ] || [ ! -f "$CHKTEX" ]; then
  echo "chktex not found in archive"
  exit 1
fi
chmod +x "$CHKTEX"

# Sample LaTeX
SAMPLE="$DIR/sample.tex"
cat > "$SAMPLE" << 'SAMPLE'
\documentclass{article}
\begin{document}
This is we that wrong.
Empty math: $$ $$ here.
\end{document}
SAMPLE

# Run
echo ""
echo "Running chktex on sample..."
"$CHKTEX" "$SAMPLE"
echo ""
echo "Binary: $CHKTEX"
echo "Add to PATH or copy to a permanent location."
echo "Try: chktex yourfile.tex"
