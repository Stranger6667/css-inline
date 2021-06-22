# Changelog

## [Unreleased]

## [0.7.2] - 2021-06-22

### Fixed

- Incorrect override of exiting `style` attribute values. [#113](https://github.com/Stranger6667/css-inline/issues/113)

### Performance

- Minor performance improvements

## [0.7.1] - 2021-06-10

### Fixed

- Ignored `style` tags when the document contains multiple of them and the `remove_style_tags: true` config option is used. [#110](https://github.com/Stranger6667/css-inline/issues/110)

## [0.7.0] - 2021-06-09

### Fixed

- Ignored selectors specificity. [#108](https://github.com/Stranger6667/css-inline/issues/108)

### Changed

- Upgrade `Pyo3` to `0.13`.

## [0.6.2] - 2021-01-28

### Fixed

- Source code distribution. It was missing the source code for the underlying Rust crate and were leading to
  a build error during `pip install css-inline` on platforms that we don't have wheels for.
  [#99](https://github.com/Stranger6667/css-inline/issues/99)

## [0.6.1] - 2020-12-07

### Added

- Python 3.9 support.

### Fixed

- Compatibility with the new `cssparser` crate version.

### Performance

- Avoid string allocations during converting `ParseError` to `InlineError`.

## [0.6.0] - 2020-11-02

### Changed

- Links to remote stylesheets are deduplicated now.
- Upgrade `Pyo3` to `0.12`.

### Performance

- Avoid setting module docstring twice
- Use `Cow` for error messages. [#87](https://github.com/Stranger6667/css-inline/issues/87)

## [0.5.0] - 2020-08-07

### Performance

- Avoid string allocation in `get_full_url`

## [0.4.0] - 2020-07-13

### Added

- Option to disable processing of "style" tags. [#45](https://github.com/Stranger6667/css-inline/issues/45)
- Option to inline additional CSS. [#45](https://github.com/Stranger6667/css-inline/issues/45)

### Changed

- Switch from `openssl` to `rustls` in `attohttpc` dependency. [#49](https://github.com/Stranger6667/css-inline/issues/49)

### Performance

- Use `ToString` trait during error handling to avoid using a formatter.

## [0.3.2] - 2020-07-09

### Fixed

- `CSSInliner` signature detection in PyCharm.

## [0.3.1] - 2020-07-07

### Changed

- Upgrade `Pyo3` to `0.11`. [#40](https://github.com/Stranger6667/css-inline/issues/40)

### Performance

- Pre-allocate the output vector.
- Reduce the average number of allocations during styles merge by a factor of 5.5x.

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

- Skip selectors that can't be parsed.
- Validate `base_url` to be a valid URL.

### Fixed

- Panic in cases when styles are applied to the currently processed "link" tags.

## 0.1.0 - 2020-06-24

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.1...HEAD
[0.7.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.0...python-v0.7.1
[0.7.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.2...python-v0.7.0
[0.6.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.1...python-v0.6.2
[0.6.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.0...python-v0.6.1
[0.6.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.5.0...python-v0.6.0
[0.5.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.4.0...python-v0.5.0
[0.4.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.2...python-v0.4.0
[0.3.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.1...python-v0.3.2
[0.3.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.0...python-v0.3.1
[0.3.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.2.0...python-v0.3.0
[0.2.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.1.0...python-v0.2.0
