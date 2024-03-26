# Changelog

## [Unreleased]

### Added

- External stylesheet caching. [#314](https://github.com/Stranger6667/css-inline/issues/314)

## [0.13.2] - 2024-03-25

### Changed

- Update `html5ever` to `0.27`.

### Fixed

- WASM package build. [#341](https://github.com/Stranger6667/css-inline/issues/341)

## [0.13.1] - 2024-03-12

### Added

- Packages for `aarch64-linux-android` & `arm-linux-androideabi`. [#336](https://github.com/Stranger6667/css-inline/issues/336)

### Fixed

- Error during loading the module on `x86_64-unknown-linux-musl`

## [0.13.0] - 2024-01-19

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

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.2...HEAD
[0.13.2]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.1...javascript-v0.13.2
[0.13.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.0...javascript-v0.13.1
[0.13.0]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.1...javascript-v0.13.0
[0.12.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.0...javascript-v0.12.1
