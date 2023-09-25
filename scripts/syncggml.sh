#!/bin/bash
set -euo pipefail
LATEST_GGML_RELEASE=$(curl -sL https://api.github.com/repos/ggerganov/llama.cpp/releases/latest | jq -er '.tag_name')
OUR_GGML_RELEASE=$(cat ./ggml-tag-current.txt 2>/dev/null || echo -n 'xx-no-current-release')

if test -z "${GITHUB_OUTPUT:-}"; then
  echo 'Empty GITHUB_OUTPUT environment variable.' >&2
  exit 1
fi
if test -z "${LATEST_GGML_RELEASE}" -o -z "${OUR_GGML_RELEASE}"; then
  echo 'Empty release string for remote or current!' >&2
  exit 1
fi
if ! echo -n "${LATEST_GGML_RELEASE}!${OUR_GGML_RELEASE}" | grep -Eiq '^[.A-Z0-9-]{2,}![.A-Z0-9-]{2,}$'; then
  echo 'Bad release format for remote or current!' >&2
  exit 1
fi
if test "${LATEST_GGML_RELEASE}" = "${OUR_GGML_RELEASE}"; then
  echo 'new_release=false' >> $GITHUB_OUTPUT
  exit 0
fi
if test "${1:-}" = "only-check"; then
  echo 'new_release=true' >> $GITHUB_OUTPUT
  exit 0
fi
echo "New release tag. Latest [${LATEST_GGML_RELEASE}], ours: [${OUR_GGML_RELEASE}]"
git clone --depth 300 --single-branch https://github.com/ggerganov/llama.cpp ggml-repo && \
  ( cd ggml-repo && git checkout "$LATEST_GGML_RELEASE" )
mkdir -p ggml-src/{pocs,tests,examples,scripts,common}
touch ggml-src/{pocs,tests,examples}/CMakeLists.txt
cp ggml-repo/*.{c,cpp,h,m,metal,cu} ggml-repo/CMakeLists.txt ggml-src/
cp ggml-repo/common/*.{cpp,h} ggml-repo/common/CMakeLists.txt ggml-src/common/
cp ggml-repo/scripts/build-info.{cmake,h.in} ggml-repo/scripts/LlamaConfig.cmake.in ggml-src/scripts/
git add \
  ggml-src/*.{c,cpp,h,m,metal,cu} \
  ggml-src/common/*.* \
  ggml-src/scripts/build-info.* ggml-src/scripts/LlamaConfig.cmake.in \
  ggml-src/CMakeLists.txt ggml-src/{tests,pocs,examples}/CMakeLists.txt

if test -z "`git status --untracked=no --porcelain`"; then
  # New release tag but no relevant changes, so nothing to do.
  exit 0
fi

STAMP=$(date -u '+%y%m%d%H%M')
VERSION="${STAMP}.0.0+llamacpp-release.${LATEST_GGML_RELEASE}"

sed -i "s@^version =.*@version = \"${VERSION}\"@" ./Cargo.toml

if ! test -f src/lib.rs; then
  mkdir -p src/
  touch src/lib.rs
fi

# Make sure it actually builds.
cargo build
cargo test
cargo clean
cargo build --features use_cmake
cargo test --features use_cmake

echo "$VERSION" > ./VERSION.txt
echo "$OUR_GGML_RELEASE" > ./ggml-tag-previous.txt
echo "$LATEST_GGML_RELEASE" > ./ggml-tag-current.txt
git add Cargo.toml VERSION.txt ggml-tag-current.txt ggml-tag-previous.txt src/lib.rs ggml-src/build-info.h
git config user.name github-actions
git config user.email github-actions@github.com
( echo -e "[auto] Sync version ${VERSION}\n\n== Relevant log messages from source repo:\n" ; \
  cd ggml-repo && \
  git log "${OUR_GGML_RELEASE}..${LATEST_GGML_RELEASE}" -- \
    *.{c,cpp,h,m,metal,cu} CMakeLists.txt \
    scripts/build-info.{cmake,h.in} scripts/LlamaConfig.cmake.in \
    2>/dev/null || true \
) | git commit -F -
git push
echo 'new_release=true' >> $GITHUB_OUTPUT
echo "new_release_version=${VERSION}" >> $GITHUB_OUTPUT
