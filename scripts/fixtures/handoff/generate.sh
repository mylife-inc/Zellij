#!/bin/sh
set -eu
usage() {
  cat <<'HELP'
Usage: sh generate.sh [build|check|watch|view|serve] [zel options]
  build  Regenerate site/ (default); --offline uses the cached renderer.
  check  Validate source without generating a website; try --deep --strict.
  watch  Repeat validation when source changes.
  view   Build, serve and open a browser; add --watch for live reload.
  serve  Serve the existing site/ without rebuilding it.
HELP
}
MODE=build
case "${1:-}" in
  build|check|watch|view|serve) MODE=$1; shift ;;
  -h|--help|help) usage; exit 0 ;;
  ''|-*) ;;
  *) printf 'Unknown operation: %s\n' "$1" >&2; usage >&2; exit 2 ;;
esac
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
case "$(uname -s):$(uname -m)" in
  Darwin:arm64) ZEL="$ROOT/zel-macos-arm64" ;;
  Darwin:x86_64) ZEL="$ROOT/zel-macos-intel" ;;
  *) ZEL=$(command -v zel || true)
     test -n "$ZEL" || { echo 'Install zel for this operating system; see README.md.' >&2; exit 1; } ;;
esac
test -x "$ZEL" || chmod +x "$ZEL"
case "$MODE" in
  check|watch) exec "$ZEL" "$MODE" "$ROOT/zellij" "$@" ;;
  view) exec "$ZEL" view "$ROOT/zellij" --features diagrams "$@" ;;
  serve) exec "$ZEL" serve "$ROOT/site" "$@" ;;
  build)
    "$ZEL" build "$ROOT/zellij" --features diagrams "$@"
    SHOW_READY=true
    for option in "$@"; do
      case "$option" in -h|--help) SHOW_READY=false ;; esac
    done
    if [ "$SHOW_READY" = true ]; then printf 'Ready: %s/site/index.html\n' "$ROOT"; fi
    ;;
esac
