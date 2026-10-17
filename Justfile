default:
  @just --list

# Release a component: just release <rust|c|python|java|php|ruby|javascript> 0.22.0
release COMPONENT VERSION:
  #!/usr/bin/env bash
  set -euo pipefail
  C="{{COMPONENT}}"
  VERSION="{{VERSION}}"
  [ -z "$(git status --porcelain)" ] || { echo "working tree is not clean"; exit 1; }
  case "$C" in
    rust)       CL=CHANGELOG.md;                     FILES=(crates/css-inline/Cargo.toml) ;;
    c)          CL=crates/css-inline-c/CHANGELOG.md;          FILES=(crates/css-inline-c/Cargo.toml) ;;
    python)     CL=crates/css-inline-python/CHANGELOG.md;     FILES=(crates/css-inline-python/Cargo.toml) ;;
    java)       CL=crates/css-inline-java/CHANGELOG.md; FILES=(crates/css-inline-java/Cargo.toml crates/css-inline-java/build.gradle crates/css-inline-java/README.md) ;;
    php)        CL=crates/css-inline-php/CHANGELOG.md;        FILES=(crates/css-inline-php/Cargo.toml crates/css-inline-php/stubs/css_inline.php) ;;
    ruby)       CL=crates/css-inline-ruby/CHANGELOG.md; FILES=(crates/css-inline-ruby/Cargo.toml crates/css-inline-ruby/css_inline.gemspec crates/css-inline-ruby/ext/css_inline/Cargo.toml crates/css-inline-ruby/Gemfile.lock) ;;
    javascript) CL=crates/css-inline-js/CHANGELOG.md; FILES=(crates/css-inline-js/Cargo.toml crates/css-inline-js/package.json crates/css-inline-js/js-binding.js crates/css-inline-js/wasm/package.json crates/css-inline-js/npm/*/package.json crates/css-inline-js/README.md crates/css-inline-js/wasm/README.md) ;;
    *) echo "unknown component: $C"; exit 1 ;;
  esac
  PREV=$(grep -oP "compare/${C}-v\K[0-9]+\.[0-9]+\.[0-9]+(?=\.\.\.HEAD)" "$CL")
  PREV_RE=${PREV//./\\.}
  DATE=$(date +%Y-%m-%d)
  # Bump only the package's own version line. A global replace also hits
  # dependency pins that happen to share the version (`jni = "0.21.1"`).
  for f in "${FILES[@]}"; do
    case "$f" in
      *.md|*.js)     sed -i "s/${PREV_RE}/${VERSION}/g" "$f"; continue ;;
      *Gemfile.lock) ANCHOR="css_inline .${PREV_RE}." ;;
      *)             ANCHOR="version.*${PREV_RE}" ;;
    esac
    grep -qiE "$ANCHOR" "$f" || { echo "no version line matching '${PREV}' in $f"; exit 1; }
    sed -i "0,/${ANCHOR}/Is/${PREV_RE}/${VERSION}/" "$f"
  done
  sed -i "0,/^## \[Unreleased\]$/s//## [Unreleased]\n\n## [${VERSION}] - ${DATE}/" "$CL"
  sed -i "s#compare/${C}-v${PREV_RE}\.\.\.HEAD#compare/${C}-v${VERSION}...HEAD#" "$CL"
  sed -i "/^\[Unreleased\]: /a [${VERSION}]: https://github.com/Stranger6667/css-inline/compare/${C}-v${PREV}...${C}-v${VERSION}" "$CL"
  if [ "$C" = ruby ]; then
    # The gem builds the core from crates.io; pin it to the core's current version.
    CORE=$(grep -m1 -oP '^version = "\K[^"]+' crates/css-inline/Cargo.toml)
    sed -i "/^\[dependencies.css-inline\]$/,/^version/s/^version = .*/version = \"${CORE}\"/" crates/css-inline-ruby/ext/css_inline/Cargo.toml
    grep -A1 '^\[dependencies.css-inline\]$' crates/css-inline-ruby/ext/css_inline/Cargo.toml | grep -qx "version = \"${CORE}\"" || { echo "css-inline pin not bumped to ${CORE}"; exit 1; }
    cargo update -p css-inline --manifest-path crates/css-inline-ruby/ext/css_inline/Cargo.toml
  fi
  git add -u
  git commit -m "chore(${C}): Release ${VERSION}"
  git tag "${C}-v${VERSION}"
  git push origin master
  git push origin "${C}-v${VERSION}"
