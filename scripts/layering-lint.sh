#!/usr/bin/env bash
# Layering guardrail (ADR-0002 / ADR-0003).
#
# hoforras-domain and hoforras-ports are the pure core of the hexagon. They MUST NOT depend —
# directly or transitively — on any Ruv-ecosystem crate (daa-*, rvm-*, qudag, ruvector, ruv-*).
# A violation means a vendor type has leaked into the ubiquitous language / port contracts, which
# breaks testability and isolation. This script fails the build (CI) on any violation.
set -euo pipefail

# Forbidden crate-name prefixes (anchored at the start of a cargo-tree crate name).
FORBIDDEN_REGEX='^(daa[-_]|rvm[-_]|qudag|ruvector|ruv[-_]|ruv-swarm|ruv-fann)'
PURE_CRATES=("hoforras-domain" "hoforras-ports")

# Ensure cargo is available (CI installs rustup; local sessions source ~/.cargo/env).
if ! command -v cargo >/dev/null 2>&1; then
  # shellcheck disable=SC1090
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
fi

fail=0

for crate in "${PURE_CRATES[@]}"; do
  echo "== layering-lint: ${crate} (transitive normal deps) =="
  # cargo tree lists the full normal-dependency closure; -e normal excludes dev/build deps.
  # We strip the tree glyphs and the version, leaving bare crate names.
  deps="$(cargo tree -p "${crate}" -e normal --prefix none 2>/dev/null \
    | sed -E 's/^[^a-zA-Z0-9_-]*//' \
    | awk '{print $1}' \
    | grep -v "^${crate}$" \
    | sort -u || true)"

  if echo "${deps}" | grep -E "${FORBIDDEN_REGEX}" >/dev/null 2>&1; then
    echo "  ✗ FORBIDDEN Ruv dependency found in ${crate}:"
    echo "${deps}" | grep -E "${FORBIDDEN_REGEX}" | sed 's/^/      /'
    fail=1
  else
    echo "  ✓ no Ruv-crate dependency"
  fi

  # Belt-and-suspenders: source-level check for forbidden imports.
  if grep -REn 'use[[:space:]]+(daa_|rvm_|qudag|ruvector|ruv_)' "crates/${crate}/src" 2>/dev/null; then
    echo "  ✗ FORBIDDEN Ruv import in ${crate}/src"
    fail=1
  fi
done

if [ "${fail}" -ne 0 ]; then
  echo "layering-lint: FAILED (ADR-0002/0003 violation)"
  exit 1
fi
echo "layering-lint: PASSED — domain and ports are Ruv-free (ADR-0002/0003)."
