# Changelog

## [Unreleased]

### Added

- Support for the `data-css-inline="keep"` attribute to enforce keeping the `style` tag.

### Fixed

- Lookups for previous / next siblings, affecting selectors like `nth-child`. [#324](https://github.com/Stranger6667/css-inline/issues/324)

### Performance

- Avoid using binary search on attributes.

## [0.12.1] - 2023-12-31

### Added

- Package for `aarch64-apple-darwin`.

### Changed

- Avoid loading additional dependencies for WASM resulting in ~6% module size reduction.

## [0.12.0] - 2023-12-28

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.0...HEAD
[0.12.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.0...javascript-v0.12.1
