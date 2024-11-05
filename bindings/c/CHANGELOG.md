# Changelog

## [Unreleased]

### Changed

- Bump MSRV to `1.70`.

### Fixed

- Replace double quotes when merging styles. [#392](https://github.com/Stranger6667/css-inline/issues/392)

## [0.14.1] - 2024-04-27

### Fixed

- Precedence of element styles over other styles. [#364](https://github.com/Stranger6667/css-inline/issues/364)

## [0.14.0] - 2024-04-01

### Added

- External stylesheet caching. [#314](https://github.com/Stranger6667/css-inline/issues/314)
- Inlining to HTML fragments. [#335](https://github.com/Stranger6667/css-inline/issues/335)

### Changed

- Update `html5ever` to `0.27`.

## [0.13.0] - 2024-01-19

### Added

- Support for the `data-css-inline="keep"` attribute to enforce keeping the `style` tag.

### Fixed

- Lookups for previous / next siblings, affecting selectors like `nth-child`. [#324](https://github.com/Stranger6667/css-inline/issues/324)

### Performance

- Avoid using binary search on attributes.

## [0.12.0] - 2023-12-28

### Changed

- Necessary updates based on the main crate changes. There are no user-facing changes.

## 0.11.3 - 2023-12-14

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/c-v0.14.1...HEAD
[0.14.1]: https://github.com/Stranger6667/css-inline/compare/c-v0.14.0...c-v0.14.1
[0.14.0]: https://github.com/Stranger6667/css-inline/compare/c-v0.13.0...c-v0.14.0
[0.13.0]: https://github.com/Stranger6667/css-inline/compare/c-v0.12.0...c-v0.13.0
[0.12.0]: https://github.com/Stranger6667/css-inline/compare/c-v0.11.3...c-v0.12.0
