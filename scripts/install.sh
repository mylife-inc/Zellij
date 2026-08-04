#!/bin/sh
#
# Install `zel`, the Zellij CLI.
#
#     curl -fsSL https://zellij.shebka.net/install.sh | sh
#
# Binaries live in a public releases repository, because Zellij's own
# repository is private and a private repository's release assets are not
# downloadable without a token. Nothing here needs one.
#
#   ZEL_VERSION      pin a version, e.g. v0.1.0. Default: the latest.
#   ZEL_INSTALL_DIR  where to put the binary. Default: ~/.local/bin.
#
# POSIX sh on purpose: this runs on whatever the machine has.

set -eu

REPO="${ZEL_REPO:-mylife-inc/releases}"
# Releases in the shared repository are named for the project that produced
# them, so several private repositories can publish into one public one.
SCOPE="${ZEL_SCOPE:-zellij}"
INSTALL_DIR="${ZEL_INSTALL_DIR:-$HOME/.local/bin}"

say()  { printf '\033[1;36m▸\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m✗\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "$1 is required and was not found."; }

need uname
need tar
command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1 ||
  die "Either curl or wget is required."

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1"
  else
    wget -qO- "$1"
  fi
}

fetch_to() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$2" "$1"
  else
    wget -qO "$2" "$1"
  fi
}

# ------------------------------------------------------------------ platform

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux)  os_part="unknown-linux-gnu" ;;
  Darwin) os_part="apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*)
    die "Windows is supported, but not by this script. Download the .zip from
    https://github.com/$REPO/releases and put zel.exe on your PATH." ;;
  *) die "Unsupported operating system: $os" ;;
esac

case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) die "Unsupported architecture: $arch" ;;
esac

target="${arch_part}-${os_part}"

# ------------------------------------------------------------------- version

if [ -n "${ZEL_VERSION:-}" ]; then
  version="$ZEL_VERSION"
else
  say "Finding the latest release"
  # The shared repository holds releases for several projects, so `latest` is
  # not ours. Take the newest tag carrying our scope.
  version="$(
    fetch "https://api.github.com/repos/$REPO/releases?per_page=100" |
      sed -n 's/.*"tag_name": *"'"$SCOPE"'-\(v[^"]*\)".*/\1/p' |
      head -1
  )" || true

  [ -n "$version" ] || die "Could not find a $SCOPE release in $REPO.
    Set ZEL_VERSION to install a specific one."
fi

tag="${SCOPE}-${version}"
archive="zel-${version}-${target}.tar.gz"
base="https://github.com/$REPO/releases/download/$tag"

# ------------------------------------------------------------------ download

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

say "Downloading zel $version for $target"
fetch_to "$base/$archive" "$tmp/$archive" ||
  die "No build for $target in $tag.
    See https://github.com/$REPO/releases/tag/$tag"

# A checksum that is never verified is decoration. If one is published, it has
# to match; if the download is served over a compromised connection, this is
# the only thing standing in the way.
if fetch_to "$base/$archive.sha256" "$tmp/$archive.sha256" 2>/dev/null; then
  say "Verifying"
  expected="$(cut -d' ' -f1 < "$tmp/$archive.sha256")"

  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$tmp/$archive" | cut -d' ' -f1)"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$tmp/$archive" | cut -d' ' -f1)"
  else
    actual=""
    warn "No sha256sum or shasum — the download was not verified."
  fi

  if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
    die "Checksum mismatch.
    expected $expected
    actual   $actual
    Nothing was installed."
  fi
else
  warn "No checksum published for $archive — the download was not verified."
fi

# ------------------------------------------------------------------- install

tar -xzf "$tmp/$archive" -C "$tmp"
[ -f "$tmp/zel" ] || die "The archive did not contain a zel binary."

mkdir -p "$INSTALL_DIR"
# Move into place rather than copy over a running binary: replacing an
# executable that is currently executing fails on some systems, and `zel
# upgrade` calls this script.
mv "$tmp/zel" "$INSTALL_DIR/zel.new"
chmod +x "$INSTALL_DIR/zel.new"
mv "$INSTALL_DIR/zel.new" "$INSTALL_DIR/zel"

say "Installed $INSTALL_DIR/zel"

# ---------------------------------------------------------------------- PATH

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "$INSTALL_DIR is not on your PATH."
    printf '\n    Add this to your shell profile:\n\n'
    printf '      export PATH="%s:$PATH"\n\n' "$INSTALL_DIR"
    ;;
esac

"$INSTALL_DIR/zel" --version || true

cat <<'EOF'

  Next:

    zel new docs my-docs     a content folder that already validates
    zel dev my-docs          serve it, and watch it
    zel --help               everything else

  https://zellij.shebka.net/guide/quickstart
EOF
