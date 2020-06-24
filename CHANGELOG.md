# Changelog

## [Unreleased]

### Added

- `CSSInliner` and customization options. [#9](https://github.com/Stranger6667/css-inline/issues/9)
- Option to remove "style" tags. [#11](https://github.com/Stranger6667/css-inline/issues/11)
- `CSSInliner::compact()` constructor for producing smaller HTML output.
- `CSSInliner.inline_to` that writes the output to a generic writer. [#24](https://github.com/Stranger6667/css-inline/issues/24)

### Changed

- Improved error messages. [#27](https://github.com/Stranger6667/css-inline/issues/27)

### Fixed

- Ignore `@media` queries, since they can not be inlined. [#7](https://github.com/Stranger6667/css-inline/issues/7)

### Performance

- Improve performance for merging new styles in existing "style" attributes.

## 0.1.0 - 2020-06-22

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/v0.1.0...HEAD
