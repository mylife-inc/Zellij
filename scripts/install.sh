#!/bin/sh
#
# Install `zel`, the Zellij CLI.
#
#     curl -fsSL https://raw.githubusercontent.com/mylife-inc/releases/main/zellij/install.sh | sh
#
# Binaries live in the shared public releases repository. No token is required.
#
#   ZEL_VERSION      pin a version, e.g. v0.1.0. Default: the latest.
#   ZEL_INSTALL_DIR  where to put the binary. Default: ~/.local/bin.
#   ZEL_RELEASE_BASE_URL  mirror directory of assets; requires ZEL_VERSION.
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

# Authenticated whenever a token is around, because the anonymous GitHub API
# is rate-limited *by IP*, and CI runners share heavily-used address pools.
# Four jobs of one matrix installing at once is enough to hit it: one gets
# through and the rest get 403, on a public repository that needs no
# credential to read. GITHUB_TOKEN is injected into every Actions job, so this
# costs nothing to arrange and lifts the limit from 60 requests an hour to 1000.
fetch() {
  _tok="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
  if command -v curl >/dev/null 2>&1; then
    if [ -n "$_tok" ]; then
      curl -fsSL -H "Authorization: Bearer $_tok" "$1"
    else
      curl -fsSL "$1"
    fi
  else
    if [ -n "$_tok" ]; then
      wget -qO- --header="Authorization: Bearer $_tok" "$1"
    else
      wget -qO- "$1"
    fi
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
    die "For native Windows use install.ps1 from PowerShell.
    Under WSL, install.sh selects a Linux binary." ;;
  *) die "Unsupported operating system: $os" ;;
esac

case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) die "Unsupported architecture: $arch" ;;
esac

target="${arch_part}-${os_part}"

# ------------------------------------------------------------------- version

if [ -n "${ZEL_RELEASE_BASE_URL:-}" ] && [ -z "${ZEL_VERSION:-}" ]; then
  die "ZEL_RELEASE_BASE_URL requires an explicit ZEL_VERSION."
fi

if [ -n "${ZEL_VERSION:-}" ]; then
  version="$ZEL_VERSION"
else
  say "Finding the latest release"
  # The shared repository holds releases for several projects, so `latest` is
  # not ours. Take the newest tag carrying our scope.
  #
  # Pre-releases are skipped: somebody running an install one-liner is asking
  # for the stable version. A pinned ZEL_VERSION still installs whatever it
  # names, which is how a release candidate gets tested.
  #
  # The filtering is in the pattern rather than a second `grep -v -- '-'`,
  # because a lone dash as a grep argument is exactly the sort of thing one
  # implementation somewhere reads as an option. `v0.1.0-rc.1` does not match
  # here: the closing quote has to follow the digits.
  # The request and the parse are separate, and the request's failure is not
  # swallowed. `|| true` on the whole pipeline turned every transport failure
  # into "found no stable release" — a sentence about the wrong thing. A 403
  # from rate limiting reported a missing release, and the release was there.
  if ! releases="$(fetch "https://api.github.com/repos/$REPO/releases?per_page=100")"; then
    die "Could not ask GitHub which $SCOPE releases exist.
    The request to api.github.com failed — commonly rate limiting, which is
    per-IP and applies to anonymous requests even on a public repository.
    Set GITHUB_TOKEN (Actions injects one) to raise the limit, or pin a
    version with ZEL_VERSION to skip the lookup entirely."
  fi

  version="$(
    printf '%s' "$releases" |
      sed -n 's/.*"tag_name": *"'"$SCOPE"'-\(v[0-9][0-9.]*\)".*/\1/p' |
      head -1
  )"

  [ -n "$version" ] || die "Found no stable $SCOPE release in $REPO.
    There may be a pre-release: see
    https://github.com/$REPO/releases
    and install it with ZEL_VERSION=v0.0.0-rc.1"
fi

printf '%s\n' "$version" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$' ||
  die "Invalid version; expected v0.2.0 or v0.2.0-rc.1."
tag="${SCOPE}-${version}"
archive="zel-${version}-${target}.tar.gz"
base="${ZEL_RELEASE_BASE_URL:-https://github.com/$REPO/releases/download/$tag}"
base="${base%/}"
case "$base" in
  *'?'*|*'#'*|*'@'*) die "Release asset URLs must not contain credentials, a query or a fragment." ;;
esac
case "$base" in
  https://*|http://127.0.0.1:*|http://localhost:*|http://\[::1\]:*) ;;
  *) die "Release asset URLs must be HTTPS (or loopback HTTP)." ;;
esac
case "$base" in
  http://*)
    authority="${base#http://}"
    authority="${authority%%/*}"
    port="${authority##*:}"
    case "$port" in ''|*[!0-9]*) die "Release asset URLs must use a numeric loopback port." ;; esac
    ;;
esac

# ------------------------------------------------------------------ download

tmp="$(mktemp -d)"
install_stage=""
cleanup() {
  [ -z "$install_stage" ] || rm -f "$install_stage"
  rm -rf "$tmp"
}
trap cleanup EXIT
trap 'exit 1' INT TERM

say "Downloading zel $version for $target"
fetch_to "$base/$archive" "$tmp/$archive" ||
  die "No build for $target in $tag.
    See https://github.com/$REPO/releases/tag/$tag"

# A checksum that is never verified is decoration. If one is published, it has
# to match; if the download is served over a compromised connection, this is
# the only thing standing in the way.
#
# So every path that cannot verify stops, rather than warning and carrying on.
# This is a script people run as `curl … | sh`, which means whoever can
# interfere with the download can also interfere with the checksum — and the
# easiest way to do that is not to forge it but to make the request for it
# fail. A warning is not a defence against that. It is the attack succeeding
# with a note attached.
#
# Nothing legitimate is lost by refusing: every release publishes checksums, so
# a missing one means the release is broken or the response is not the one that
# was published. Both are reasons to stop.
#
# Two formats are accepted, because two things produce releases and they chose
# differently. `SHA256SUMS` — one file listing every archive — is what the local
# release path writes and what CodeSeal publishes. A `.sha256` beside each
# archive is what the GitHub Actions workflow writes.
#
# Reading only the second is what broke the first CI install of `zel`: the
# release was complete and correct, the installer asked for a file that release
# had no reason to contain, and it failed closed with a message about a broken
# release. It failed the right way about the wrong thing.
#
# SHA256SUMS is tried first: it is the conventional form, and a release that has
# both is a release in transition rather than a reason to prefer the older one.
# Finding the expected hash and checking it are separate, and the check is
# written ONCE, outside the branches.
#
# The first version of this put the comparison inside the `.sha256` branch and
# added the SHA256SUMS branch beside it. That branch found the hash, announced
# "Verifying", and then fell out of the `if` — nothing hashed the download,
# nothing compared anything, and the install proceeded. A fail-open path in the
# one place that must fail closed, and it looked correct: it printed the
# reassuring line and installed a working binary.
#
# The installation test passed *because* of the bug. Only a deliberately
# corrupted archive shows the difference, which is what the test below does.
if [ "${ZEL_INSECURE_SKIP_CHECKSUM:-}" = "1" ]; then
  warn "ZEL_INSECURE_SKIP_CHECKSUM=1 — installing without verifying the download."
else
  # ---- find the expected hash -------------------------------------------
  #
  # Two formats, because two things produce releases and they chose
  # differently. `SHA256SUMS` — one file listing every archive — is what the
  # local release path writes and what CodeSeal publishes. A `.sha256` beside
  # each archive is what the GitHub Actions workflow writes. Reading only the
  # second is what broke the first CI install of `zel`.
  expected=""
  if fetch_to "$base/SHA256SUMS" "$tmp/SHA256SUMS" 2>/dev/null; then
    # `shasum` writes "<hash>  ./<name>" or "<hash>  <name>"; accept either, and
    # match on the basename so a leading ./ cannot cause a false miss.
    expected="$(awk -v want="$archive" '
      { name = $2; sub(/^\.\//, "", name); if (name == want) { print $1; exit } }
    ' "$tmp/SHA256SUMS")"
    [ -n "$expected" ] && say "Verifying against SHA256SUMS"
  fi

  if [ -z "$expected" ] && fetch_to "$base/$archive.sha256" "$tmp/$archive.sha256" 2>/dev/null; then
    expected="$(cut -d' ' -f1 < "$tmp/$archive.sha256")"
    [ -n "$expected" ] && say "Verifying"
  fi

  if [ -z "$expected" ]; then
    die "No checksum could be fetched for $archive, so the download cannot be
    verified. Looked for SHA256SUMS and for $archive.sha256; every release
    publishes one of them, so this means either the release is incomplete or
    the response was not the published one. Nothing was installed.

    To install anyway: ZEL_INSECURE_SKIP_CHECKSUM=1"
  fi

  # ---- and check it ------------------------------------------------------
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$tmp/$archive" | cut -d' ' -f1)"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$tmp/$archive" | cut -d' ' -f1)"
  else
    die "Neither sha256sum nor shasum is available, so the download cannot be
    verified. Install one, or re-run with ZEL_INSECURE_SKIP_CHECKSUM=1 if you
    accept an unverified binary."
  fi

  if [ "$actual" != "$expected" ]; then
    die "Checksum mismatch.
    expected $expected
    actual   $actual
    Nothing was installed."
  fi
fi

# ------------------------------------------------------------------- install

# Take one named member out of the archive, and check it is what it claims.
#
# A tarball is a list of paths chosen by whoever built it, and `tar -x` honours
# that list — including symlinks, and including members that unpack outside the
# directory you pointed it at. Naming `zel` means nothing else in the archive is
# written to disk at all, which is the difference between extracting an archive
# and trusting one.
#
# The `-L` check is the other half. A member named `zel` can be a symlink, and
# a symlink survives `chmod +x` and `mv` perfectly well — what lands on your
# PATH would be a pointer at whatever the archive chose.
#
# `--no-same-owner` is not in POSIX tar, so a shell that has not got it falls
# through to the plain form rather than failing to install.
tar --no-same-owner -xzf "$tmp/$archive" -C "$tmp" zel 2>/dev/null ||
  tar -xzf "$tmp/$archive" -C "$tmp" zel 2>/dev/null ||
  die "The archive did not contain a zel binary."

if [ ! -f "$tmp/zel" ] || [ -L "$tmp/zel" ]; then
  die "The archive's zel is not a regular file. Nothing was installed."
fi

# Validate the executable before replacing a working installation.
chmod +x "$tmp/zel"
reported="$("$tmp/zel" --version)" || die "The downloaded binary cannot run. Nothing was installed."
[ "$reported" = "zel ${version#v}" ] || die "Binary version does not match $version. Nothing was installed."
if [ -L "$INSTALL_DIR/zel" ] || { [ -e "$INSTALL_DIR/zel" ] && [ ! -f "$INSTALL_DIR/zel" ]; }; then
  die "The existing zel must be a regular file. Nothing was installed."
fi
mkdir -p "$INSTALL_DIR"
install_stage="$(mktemp "$INSTALL_DIR/.zel-install.XXXXXX")"
# Rename into place on the destination filesystem; never truncate the old inode.
cp "$tmp/zel" "$install_stage"
chmod +x "$install_stage"
mv -f "$install_stage" "$INSTALL_DIR/zel"
install_stage=""

say "Installed $INSTALL_DIR/zel"

# ---------------------------------------------------------------------- PATH

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "$INSTALL_DIR is not on your PATH."
    printf '\n    Add this to your shell profile:\n\n'
    # shellcheck disable=SC2016 # Print the user's future PATH expansion literally.
    printf '      export PATH="%s:$PATH"\n\n' "$INSTALL_DIR"
    ;;
esac

"$INSTALL_DIR/zel" --version

cat <<'EOF'

  Next:

    zel new portable my-docs   a documentation site that already validates
    zel build my-docs          generate HTML that opens directly from disk
    zel view my-docs --watch   build, preview and reload edits
    zel --help               everything else

  https://zellij.shebka.net/guide/quickstart
EOF
