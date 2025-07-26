# Changelog

## [Unreleased]

### Added

- `keep_at_rules` option to keep all "at-rules" (ones starting with `@`) inside a "style" element [#265](https://github.com/Stranger6667/css-inline/issues/265)

### Performance

- Avoid unnecessary check for double quotes.

## [0.16.0] - 2025-07-16

### Changed

- Bump MSRV to `1.75`.
- Update `selectors` to `0.30`.
- Update `html5ever` to `0.35`.
- Fallback to WASM when native bindings are missing.

### Fixed

- Expose `StylesheetCache` interface, `cache` config option, and `version` function.
- Ignored `!important` that has insignificant whitespace after it.

### Performance

- Use interned string to compare `style` element name.
- Only check the value suffix for `!important`.

## [0.15.0] - 2025-06-24

### Changed

- Bump MSRV to `1.71.1`.
- update `cssparser` to `0.35`.
- update `html5ever` to `0.31`.
- update `selectors` to `0.28`.

### Performance

- Slightly improve performance of HTML serialization.
- Avoid creating unused caches.

## [0.14.3] - 2024-11-14

### Added

- Packages for `win32-arm64-msvc`. [#397](https://github.com/Stranger6667/css-inline/issues/397)

### Fixed

- Prioritize `!important` rules when computing element styles. [#398](https://github.com/Stranger6667/css-inline/pull/398)

## [0.14.2] - 2024-11-11

### Changed

- Bump MSRV to `1.70`.

### Fixed

- Build on `linux-arm-gnueabihf`
- Replace double quotes when merging styles. [#392](https://github.com/Stranger6667/css-inline/issues/392)

## [0.14.1] - 2024-04-27

### Fixed

- Precedence of element styles over other styles. [#364](https://github.com/Stranger6667/css-inline/issues/364)

## [0.14.0] - 2024-04-01

### Added

- External stylesheet caching. [#314](https://github.com/Stranger6667/css-inline/issues/314)
- Inlining to HTML fragments. [#335](https://github.com/Stranger6667/css-inline/issues/335)

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

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.16.0...HEAD
[0.16.0]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.15.0...javascript-v0.16.0
[0.15.0]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.14.3...javascript-v0.15.0
[0.14.3]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.14.2...javascript-v0.14.3
[0.14.2]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.14.1...javascript-v0.14.2
[0.14.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.14.0...javascript-v0.14.1
[0.14.0]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.2...javascript-v0.14.0
[0.13.2]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.1...javascript-v0.13.2
[0.13.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.13.0...javascript-v0.13.1
[0.13.0]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.1...javascript-v0.13.0
[0.12.1]: https://github.com/Stranger6667/css-inline/compare/javascript-v0.12.0...javascript-v0.12.1
