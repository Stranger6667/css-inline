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
    rust)       CL=CHANGELOG.md;                     FILES=(css-inline/Cargo.toml) ;;
    c)          CL=bindings/c/CHANGELOG.md;          FILES=(bindings/c/Cargo.toml) ;;
    python)     CL=bindings/python/CHANGELOG.md;     FILES=(bindings/python/Cargo.toml) ;;
    java)       CL=bindings/java/CHANGELOG.md;       FILES=(bindings/java/Cargo.toml bindings/java/build.gradle bindings/java/README.md) ;;
    php)        CL=bindings/php/CHANGELOG.md;        FILES=(bindings/php/Cargo.toml bindings/php/stubs/css_inline.php) ;;
    ruby)       CL=bindings/ruby/CHANGELOG.md;       FILES=(bindings/ruby/css_inline.gemspec bindings/ruby/ext/css_inline/Cargo.toml bindings/ruby/Gemfile.lock) ;;
    javascript) CL=bindings/javascript/CHANGELOG.md; FILES=(bindings/javascript/Cargo.toml bindings/javascript/package.json bindings/javascript/wasm/package.json bindings/javascript/npm/*/package.json) ;;
    *) echo "unknown component: $C"; exit 1 ;;
  esac
  PREV=$(grep -oP "compare/${C}-v\K[0-9]+\.[0-9]+\.[0-9]+(?=\.\.\.HEAD)" "$CL")
  PREV_RE=${PREV//./\\.}
  DATE=$(date +%Y-%m-%d)
  # Bump only the package's own version line. A global replace also hits
  # dependency pins that happen to share the version (`jni = "0.21.1"`).
  for f in "${FILES[@]}"; do
    case "$f" in
      *.md)          sed -i "s/${PREV_RE}/${VERSION}/g" "$f"; continue ;;
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
    cargo update -p css-inline --manifest-path bindings/ruby/Cargo.toml
    cp bindings/ruby/Cargo.lock bindings/ruby/ext/css_inline/Cargo.lock
  fi
  git add -u
  git commit -m "chore(${C}): Release ${VERSION}"
  git tag "${C}-v${VERSION}"
  git push origin master
  git push origin "${C}-v${VERSION}"
