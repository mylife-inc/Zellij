# shellcheck shell=bash
#
# Zellij development commands.
#
#     source scripts/app.sh
#     zhelp
#
# Sourced rather than run, so the commands are shell functions: they can change
# your directory, export variables, and be tab-completed. Every one of them is
# something that would otherwise be a paragraph in a README nobody reads twice.

ZELLIJ_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")/.." && pwd)"
export ZELLIJ_ROOT

# Where the public artefacts go.
export ZELLIJ_REGISTRY="${ZELLIJ_REGISTRY:-ghcr.io}"
export ZELLIJ_OWNER="${ZELLIJ_OWNER:-mylife-inc}"
export ZELLIJ_IMAGE="${ZELLIJ_REGISTRY}/${ZELLIJ_OWNER}/zellij-base"
export ZELLIJ_SITE_DOMAIN="${ZELLIJ_SITE_DOMAIN:-zellij.shebka.net}"

# Colima puts its socket somewhere Docker Desktop's default context does not
# look. Finding it here saves the "is the daemon down?" conversation, which has
# happened once already and was wrong both times.
if [ -z "${DOCKER_HOST:-}" ] && [ -S "$HOME/.colima/default/docker.sock" ]; then
  export DOCKER_HOST="unix://$HOME/.colima/default/docker.sock"
fi

_zsay()  { printf '\033[1;36m▸\033[0m %s\n' "$*"; }
_zwarn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }
_zdie()  { printf '\033[1;31m✗\033[0m %s\n' "$*" >&2; return 1; }

# --------------------------------------------------------------------- help

zhelp() {
  cat <<'EOF'
Zellij development commands

  Building
    zbuild              Engine, then the CLI
    zbuild-engine       The npm package
    zbuild-cli          The Rust binary (debug)
    zbuild-cli-release  The Rust binary (release, stripped)

  Checking
    zcheck              Everything CI runs, in CI's order
    ztest               Unit tests, engine and CLI
    ze2e                The showcase end-to-end suite
    zschema             Regenerate the CLI's embedded schema
    zdrift              Fail if the embedded schema has fallen behind

  Running
    zdev <example>      Development server for showcase | docs | payos
    zdocs               Zellij's own documentation site

  Images
    zimage [features]   Build the base image. features: none | maths | diagrams | all
    zimage-all          Build all four variants, the way CI publishes them
    zimage-push [tag]   Push it to GHCR
    zsite <content> [tag] [features]
                        Build a site image. features: maths | diagrams | all
    zimage-size         What the images weigh, and where it goes

  Publishing
    zrelease <version>  Cut a release: checks, tags, and hands the rest to CI
    zrelease-watch      Follow the release CI is running
    znetlify            Build and deploy the docs site by hand

  Housekeeping
    zroot               cd to the repository root
    zclean              Remove build output
    zdoctor             Check the tools this needs
EOF
}

# ----------------------------------------------------------------- building

zroot() { cd "$ZELLIJ_ROOT" || return 1; }

zbuild-engine() {
  _zsay "Building @shebka/zellij"
  (cd "$ZELLIJ_ROOT" && npm run build --workspace @shebka/zellij)
}

zbuild-cli() {
  _zsay "Building zel (debug)"
  (cd "$ZELLIJ_ROOT/cli" && cargo build)
}

zbuild-cli-release() {
  _zsay "Building zel (release)"
  (cd "$ZELLIJ_ROOT/cli" && cargo build --release) || return 1
  ls -lh "$ZELLIJ_ROOT/cli/target/release/zel" | awk '{print "  " $9 "  " $5}'
}

zbuild() { zbuild-engine && zbuild-cli; }

# ----------------------------------------------------------------- checking

zschema() {
  # The CLI compiles these in, so they are regenerated from the engine rather
  # than edited. `zdrift` is what stops them being forgotten.
  _zsay "Regenerating the CLI's embedded schema"
  local bin="$ZELLIJ_ROOT/engine/packages/zellij/bin/zellij.mjs"
  local out="$ZELLIJ_ROOT/cli/crates/zellij-core/generated"
  node "$bin" context "$out/schema.json" --json &&
    node "$bin" context "$out/context.md"
}

zdrift() {
  zschema >/dev/null || return 1
  if git -C "$ZELLIJ_ROOT" diff --quiet -- cli/crates/zellij-core/generated; then
    _zsay "Embedded schema matches the engine"
  else
    _zwarn "The engine's schemas have moved and the CLI's copy has not."
    git -C "$ZELLIJ_ROOT" diff --stat -- cli/crates/zellij-core/generated
    return 1
  fi
}

ztest() {
  (cd "$ZELLIJ_ROOT" && npx vitest run --root engine/packages/zellij) &&
    (cd "$ZELLIJ_ROOT/cli" && cargo test --workspace)
}

ze2e() { (cd "$ZELLIJ_ROOT/examples/showcase" && npm run build && npx playwright test); }

zcheck() {
  _zsay "Typecheck"      && (cd "$ZELLIJ_ROOT" && npm run typecheck) &&
  _zsay "Engine tests"   && (cd "$ZELLIJ_ROOT" && npx vitest run --root engine/packages/zellij) &&
  _zsay "Schema drift"   && zdrift &&
  _zsay "CLI format"     && (cd "$ZELLIJ_ROOT/cli" && cargo fmt --all --check) &&
  _zsay "CLI lint"       && (cd "$ZELLIJ_ROOT/cli" && cargo clippy --workspace --all-targets -- -D warnings) &&
  _zsay "CLI tests"      && (cd "$ZELLIJ_ROOT/cli" && cargo test --workspace) &&
  _zsay "All green"
}

# ------------------------------------------------------------------ running

zdev() {
  local example="${1:-showcase}"
  [ -d "$ZELLIJ_ROOT/examples/$example" ] || _zdie "No example called $example" || return 1
  (cd "$ZELLIJ_ROOT/examples/$example" && npm run dev)
}

zdocs() { zdev zellij-docs; }

# ------------------------------------------------------------------- images

zimage() {
  local features="${1:-none}"
  local tag="${2:-local}"

  _zsay "Building the base image (features: $features)"
  (cd "$ZELLIJ_ROOT" && docker build \
    -f docker/base.Dockerfile \
    --build-arg "ZELLIJ_FEATURES=$features" \
    -t "zellij-base:$tag" \
    -t "${ZELLIJ_IMAGE}:$tag" .) || return 1

  zimage-size
}

zimage-all() {
  # The four CI publishes. They differ only in which optional dependencies the
  # generated manifest asks for, so they are nearly the same size — the saving
  # lands on whoever builds a site, not on whoever pulls the image.
  local ok=0
  zimage none     latest    || ok=1
  zimage maths    maths     || ok=1
  zimage diagrams diagrams  || ok=1
  zimage all      all       || ok=1
  return $ok
}

zimage-push() {
  local tag="${1:-latest}"

  docker image inspect "${ZELLIJ_IMAGE}:$tag" >/dev/null 2>&1 ||
    _zdie "No ${ZELLIJ_IMAGE}:$tag locally. Run: zimage <features> $tag" || return 1

  # GHCR wants a token, not a password. A classic PAT with write:packages, or
  # GITHUB_TOKEN inside Actions.
  [ -n "${GITHUB_TOKEN:-}" ] ||
    _zdie "GITHUB_TOKEN is not set (needs write:packages)" || return 1

  _zsay "Signing in to $ZELLIJ_REGISTRY as $ZELLIJ_OWNER"
  echo "$GITHUB_TOKEN" | docker login "$ZELLIJ_REGISTRY" -u "$ZELLIJ_OWNER" --password-stdin || return 1

  _zsay "Pushing ${ZELLIJ_IMAGE}:$tag"
  docker push "${ZELLIJ_IMAGE}:$tag"
}

zsite() {
  local content="${1:-examples/payos/content}"
  local tag="${2:-zellij-site:local}"
  local features="${3:-}"

  _zsay "Building a site image from $content${features:+ (features: $features)}"
  (cd "$ZELLIJ_ROOT" && docker build \
    -f docker/site.Dockerfile \
    --build-arg ZELLIJ_IMAGE=zellij-base \
    --build-arg ZELLIJ_VERSION=local \
    --build-arg "CONTENT=$content" \
    --build-arg "FEATURES=$features" \
    -t "$tag" .)
}

zimage-size() {
  docker images --format '{{.Repository}}:{{.Tag}}\t{{.Size}}' | grep -E '^zellij-' || true

  # Where the weight actually is, rather than where it is assumed to be.
  if docker image inspect zellij-base:local >/dev/null 2>&1; then
    printf '\n  largest layers:\n'
    docker history zellij-base:local --format '{{.Size}}\t{{.CreatedBy}}' |
      grep -v '^0B' | head -5 | cut -c1-90 | sed 's/^/    /'
  fi
}

# --------------------------------------------------------------- publishing

zrelease() {
  local version="$1"
  [ -n "$version" ] || _zdie "Usage: zrelease v0.2.0" || return 1
  [[ "$version" == v* ]] || _zdie "Versions start with v — try v$version" || return 1

  command -v gh >/dev/null || _zdie "The GitHub CLI (gh) is not installed" || return 1

  git -C "$ZELLIJ_ROOT" diff --quiet && git -C "$ZELLIJ_ROOT" diff --cached --quiet ||
    _zdie "Working tree is dirty. Commit before releasing." || return 1

  git -C "$ZELLIJ_ROOT" rev-parse "$version" >/dev/null 2>&1 &&
    _zdie "$version already exists. Releases are not re-cut; pick the next one." && return 1

  # Locally first, because a tag is public the moment it is pushed and a
  # release that fails in CI has already been announced by its own tag.
  _zsay "Checking before tagging"
  zcheck || return 1

  _zsay "Tagging $version"
  git -C "$ZELLIJ_ROOT" tag -a "$version" -m "$version" || return 1
  git -C "$ZELLIJ_ROOT" push origin "$version" || return 1

  cat <<EOF

  Tagged and pushed. GitHub is now building:

    · the four container images, for amd64 and arm64  → GHCR
    · zel for five platforms, with checksums          → the release page
    · the documentation site                          → $ZELLIJ_SITE_DOMAIN

  Watch it:  zrelease-watch
EOF
}

zrelease-watch() {
  command -v gh >/dev/null || _zdie "The GitHub CLI (gh) is not installed" || return 1
  gh run watch --repo "${ZELLIJ_OWNER}/Zellij" \
    "$(gh run list --repo "${ZELLIJ_OWNER}/Zellij" --workflow Release --limit 1 --json databaseId --jq '.[0].databaseId')"
}

znetlify() {
  command -v netlify >/dev/null || _zdie "netlify-cli is not installed (npm i -g netlify-cli)" || return 1

  _zsay "Building the documentation site"
  (cd "$ZELLIJ_ROOT/examples/zellij-docs" && ZELLIJ_OUTPUT=export npm run build) || return 1

  local out="$ZELLIJ_ROOT/examples/zellij-docs/out"
  [ -d "$out" ] || _zdie "No static export at $out — is ZELLIJ_OUTPUT=export supported by this build?" || return 1

  _zsay "Deploying to $ZELLIJ_SITE_DOMAIN"
  (cd "$ZELLIJ_ROOT/examples/zellij-docs" && netlify deploy --prod --dir out)
}

# -------------------------------------------------------------- diagnostics

zdoctor() {
  local ok=0

  _check() {
    if command -v "$1" >/dev/null 2>&1; then
      printf '  \033[32m✓\033[0m %-10s %s\n' "$1" "$(${2:-true} 2>/dev/null | head -1)"
    else
      printf '  \033[31m✗\033[0m %-10s not installed%s\n' "$1" "${3:+ — $3}"
      ok=1
    fi
  }

  echo "Tools"
  _check node "node --version"
  _check npm "npm --version"
  _check cargo "cargo --version"
  _check docker "docker --version"
  _check gh "gh --version" "needed by zrelease"
  _check netlify "netlify --version" "needed by znetlify"

  echo
  echo "Docker"
  if docker info >/dev/null 2>&1; then
    printf '  \033[32m✓\033[0m daemon     %s\n' "${DOCKER_HOST:-default context}"
  else
    printf '  \033[31m✗\033[0m daemon     not reachable at %s\n' "${DOCKER_HOST:-default context}"
    printf '               Colima? try: colima start\n'
    ok=1
  fi

  echo
  echo "Repository"
  printf '  root       %s\n' "$ZELLIJ_ROOT"
  printf '  branch     %s\n' "$(git -C "$ZELLIJ_ROOT" branch --show-current 2>/dev/null)"
  printf '  image      %s\n' "$ZELLIJ_IMAGE"

  return $ok
}

zclean() {
  _zsay "Removing build output"
  rm -rf "$ZELLIJ_ROOT"/examples/*/.next "$ZELLIJ_ROOT"/examples/*/out \
         "$ZELLIJ_ROOT"/engine/packages/zellij/dist "$ZELLIJ_ROOT"/cli/target
}

_zsay "Zellij commands loaded. Run zhelp."
