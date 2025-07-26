# Changelog

## [Unreleased]

### Added

- `keep_at_rules` option [#256](https://github.com/Stranger6667/css-inline/pull/485)

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

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/java-v0.16.0...HEAD
[0.16.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.15.0...java-v0.16.0
