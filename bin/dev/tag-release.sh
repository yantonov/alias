#!/bin/sh

# Sets the version in Cargo.toml, commits it and tags that commit.
#
# The two have to move together in one commit: release.yml builds whatever the
# tag points at, and the version the binary reports comes from Cargo.toml, so a
# tag placed on a commit that still carries the old version produces a release
# that lies about itself.
#
# Nothing is pushed. The push, and publishing the draft release that the
# workflow creates, stay manual.

set -eu

cd "$(dirname "$0")/../.."

VERSION="${1:-}"

if [ -z "${VERSION}" ]; then
  echo "Usage: $(basename "$0") <version>"
  echo "Example: $(basename "$0") 0.2.8"
  exit 1
fi

if ! printf '%s' "${VERSION}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "A version looks like 1.2.3, got: ${VERSION}"
  exit 1
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [ "${BRANCH}" != "master" ]; then
  echo "Releases are cut from master, this is ${BRANCH}"
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "The working tree is not clean:"
  git status --short
  exit 1
fi

if git rev-parse -q --verify "refs/tags/${VERSION}" > /dev/null; then
  echo "Tag ${VERSION} already exists"
  exit 1
fi

# Only the package version is expected to match: dependency versions are
# written as 'name = "x.y.z"' and rust-version does not start the line with
# 'version'. If that ever stops being true, stop rather than edit the wrong one.
VERSION_LINES="$(grep -c '^version = ' Cargo.toml)"
if [ "${VERSION_LINES}" != "1" ]; then
  echo "Expected one 'version =' line in Cargo.toml, found ${VERSION_LINES}"
  exit 1
fi

CURRENT="$(grep '^version = ' Cargo.toml | sed -E 's/^version = "(.*)"$/\1/')"
if [ "${CURRENT}" = "${VERSION}" ]; then
  echo "Cargo.toml is already at ${VERSION}"
  exit 1
fi

echo "${CURRENT} -> ${VERSION}"

sed -i.bak -E "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
rm -f Cargo.toml.bak

# Brings Cargo.lock, which carries the package version too, in step - and
# refuses to tag something that does not even compile.
cargo check --quiet

git add Cargo.toml Cargo.lock
git commit -m "release ${VERSION}"
git tag -a "${VERSION}" -m "release ${VERSION}"

echo
echo "Tagged ${VERSION}. Nothing pushed yet:"
echo "  git push origin ${BRANCH} && git push origin ${VERSION}"
echo "The workflow leaves the release as a draft; publish it, otherwise the"
echo "installer keeps offering the previous one."
