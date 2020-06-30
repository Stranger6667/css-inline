# Changelog

## [Unreleased]

### Changed

- Upgrade `Pyo3` to `0.11`. [#40](https://github.com/Stranger6667/css-inline/issues/40)

## [0.3.0] - 2020-06-27

### Changed

- Remove debug symbols from the release build

### Performance

- Various performance improvements

## [0.2.0] - 2020-06-25

### Added

- Loading external stylesheets. [#8](https://github.com/Stranger6667/css-inline/issues/8)
- Option to control whether remote stylesheets should be loaded (`load_remote_stylesheets`). Enabled by default.

### Changed

- Skip selectors, that can't be parsed.
- Validate `base_url` to be a valid URL.

### Fixed

- Panic in cases when styles are applied to the currently processed "link" tags.

## 0.1.0 - 2020-06-24

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.0...HEAD
[0.3.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.2.0...python-v0.3.0
[0.2.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.1.0...python-v0.2.0
