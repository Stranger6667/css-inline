# Changelog

## [Unreleased]

## [0.20.0] - 2026-02-07

### Added

- `applyWidthAttributes` and `applyHeightAttributes` options to add dimension HTML attributes from CSS properties on supported elements (`table`, `td`, `th`, `img`). [#652](https://github.com/Stranger6667/css-inline/issues/652)

### Performance

- Skip selectors that reference non-existent classes, IDs, or tags.
- Faster HTML serialization for styles containing double quotes.

## [0.19.1] - 2026-01-23

### Changed

- Update `html5ever` to `0.38`.
- Update `selectors` to `0.35`.

### Fixed

- Inline `!important` styles being overwritten by stylesheet `!important` styles. [#637](https://github.com/Stranger6667/css-inline/issues/637)

## [0.19.0] - 2025-12-29

### Added

- `removeInlinedSelectors` option to remove selectors that were successfully inlined from `<style>` blocks.

### Changed

- Update `cssparser` to `0.36`.
- Update `html5ever` to `0.36`.
- Update `selectors` to `0.33`.

### Performance

- Use element indexes for faster selector lookups on large documents.
- Use more efficient data structures for storing element styles.

## [0.18.0] - 2025-11-01

### Added

- `minify_css` option [#12](https://github.com/Stranger6667/css-inline/issues/12)

### Changed

- Update `selectors` to `0.32`.

## [0.17.0] - 2025-07-26

### Added

- `keep_at_rules` option to keep all "at-rules" (ones starting with `@`) inside a "style" element [#265](https://github.com/Stranger6667/css-inline/issues/265)

### Performance

- Avoid unnecessary check for double quotes.
- Avoid creating an unnecessary string cache entry.
- Use a more precise estimate for the size of the number of applied styles.
- Avoid hashtable rebuilding on small hashmaps.

## [0.16.0] - 2025-07-16

### Changed

- Remove `gson` as a dependency
- Update `selectors` to `0.30`.
- Update `html5ever` to `0.35`.

### Fixed

- Ignored `!important` that has insignificant whitespace after it.

### Performance

- Use interned string to compare `style` element name.
- Only check the value suffix for `!important`.

## 0.15.0 - 2025-06-29

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/java-v0.20.0...HEAD
[0.20.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.19.1...java-v0.20.0
[0.19.1]: https://github.com/Stranger6667/css-inline/compare/java-v0.19.0...java-v0.19.1
[0.19.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.18.0...java-v0.19.0
[0.18.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.17.0...java-v0.18.0
[0.17.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.16.0...java-v0.17.0
[0.16.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.15.0...java-v0.16.0
