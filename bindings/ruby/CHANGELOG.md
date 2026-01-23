# Changelog

## [Unreleased]

### Changed

- Update `html5ever` to `0.38`.
- Update `selectors` to `0.35`.

### Fixed

- Inline `!important` styles being overwritten by stylesheet `!important` styles. [#637](https://github.com/Stranger6667/css-inline/issues/637)

## [0.19.0] - 2025-12-29

### Added

- `remove_inlined_selectors` option to remove selectors that were successfully inlined from `<style>` blocks.

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

- Update `magnus` to `0.8`.
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

- Update `selectors` to `0.30`.
- Update `html5ever` to `0.35`.

### Fixed

- Ignored `!important` that has insignificant whitespace after it.

### Performance

- Use interned string to compare `style` element name.
- Only check the value suffix for `!important`.

## [0.15.2] - 2025-06-24

### Changed

- Bump MSRV to `1.82`.

### Fixed

- Packaging issue.
- Installation on Alpine. [#394](https://github.com/Stranger6667/css-inline/pull/394)

## [0.15.1] - 2025-06-21

### Fixed

- Packaging issue.

## [0.15.0] - 2025-06-21

### Changed

- Bump MSRV to `1.71.1`.
- Update `cssparser` to `0.35`.
- Update `html5ever` to `0.31`.
- Update `selectors` to `0.28`.

### Performance

- Slightly improve performance of HTML serialization.
- Avoid creating unused caches.

### Removed

- Support for Ruby 2.7

## [0.14.3] - 2024-11-14

### Fixed

- Prioritize `!important` rules when computing element styles. [#398](https://github.com/Stranger6667/css-inline/pull/398)

## [0.14.2] - 2024-11-11

### Changed

- Update `magnus` to `0.7`.
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

- Display stylesheet location in network-related errors.

### Performance

- Optimize serialization of attributes and text nodes.

## [0.11.2] - 2023-12-09

### Performance

- Avoid iterating over non-Element nodes.
- Reuse caches for nth index selectors.

## [0.11.1] - 2023-12-09

### Changed

- Update `indexmap` to `2.1`.
- Update `cssparser` to `0.31.2`.
- Update `selectors` to `0.25`.
- Bump MSRV to `1.65`.

### Fixed

- Replace double quotes in all property values.

### Performance

- Avoid allocation when replacing double quotes in property values.

## [0.11.0] - 2023-09-26

### Added

- The `inline_style_tags` option to control whether inlining from "style" tags should be performed. [#253](https://github.com/Stranger6667/css-inline/issues/253)

### Performance

- Reuse existing attributes when creating an element during parsing.

### Changed

- Bump MSRV to `1.63`.

## [0.10.5] - 2023-08-30

### Performance

- Pre-allocate space during serialization.
- Optimized `class` attribute handling: up to 25% faster for extensive class-dependent selectors.
- Fast-path class check for shorter class attribute values.
- Use a Bloom filter to detect if an element has no given class.
- Avoid allocating a vector during selectors compilation.
- Use `FxHasher` in more cases.

### Changed

- Drop usage of `memchr`.
- Bump MSRV to `1.62.1`.

### Changed

- Set the default value for `preallocate_node_capacity` to `32` to match other the default value in other bindings.

## [0.10.4] - 2023-08-12

### Changed

- Update `magnus` to `0.6`.

### Fixed

- Applying new styles only to the first matching tag during styles merging. [#224](https://github.com/Stranger6667/css-inline/issues/224)

### Performance

- Fix under-allocating storage for intermediate CSS styles.
- Perform CSS inlining as late as possible to avoid intermediate allocations. [#220](https://github.com/Stranger6667/css-inline/issues/220)

## [0.10.3] - 2023-07-01

### Performance

- Optimized HTML serialization for a performance boost of up to 25%.

## [0.10.2] - 2023-06-25

### Changed

- Standardized the formatting of CSS declarations: now consistently using `: ` separator between properties and values.

### Performance

- Various performance improvements.

## [0.10.1] - 2023-06-18

### Performance

- Use a simpler way for HTML tree traversal.
- Avoid hashing in some cases.

## 0.10.0 - 2023-06-17

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.19.0...HEAD
[0.19.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.18.0...ruby-v0.19.0
[0.18.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.17.0...ruby-v0.18.0
[0.17.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.16.0...ruby-v0.17.0
[0.16.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.15.2...ruby-v0.16.0
[0.15.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.15.1...ruby-v0.15.2
[0.15.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.15.0...ruby-v0.15.1
[0.15.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.14.3...ruby-v0.15.0
[0.14.3]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.14.2...ruby-v0.14.3
[0.14.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.14.1...ruby-v0.14.2
[0.14.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.14.0...ruby-v0.14.1
[0.14.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.13.0...ruby-v0.14.0
[0.13.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.12.0...ruby-v0.13.0
[0.12.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.2...ruby-v0.12.0
[0.11.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.1...ruby-v0.11.2
[0.11.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.0...ruby-v0.11.1
[0.11.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.5...ruby-v0.11.0
[0.10.5]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.4...ruby-v0.10.5
[0.10.4]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.3...ruby-v0.10.4
[0.10.3]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.2...ruby-v0.10.3
[0.10.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.1...ruby-v0.10.2
[0.10.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.0...ruby-v0.10.1
