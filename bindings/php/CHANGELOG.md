# Changelog

## [Unreleased]

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

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/php-v0.19.1...HEAD
[0.19.1]: https://github.com/Stranger6667/css-inline/compare/php-v0.19.0...php-v0.19.1
