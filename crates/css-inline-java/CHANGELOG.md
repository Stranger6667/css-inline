# Changelog

## [Unreleased]

### Performance

- Find top-level commas in selector lists with a fast scan for the common case.
- Look up elements by a hash of their ID, class, or tag name instead of interning it.
- Avoid a reallocation and two attribute copies per element while building the document tree.
- Return the output buffer from `inline` and `inline_fragment` without copying it.
- Build the remaining `<style>` content for `remove_inlined_selectors` without intermediate copies.
- Skip selectors with pseudo-classes that never match during inlining, such as `:hover`, before matching them.
- Use an index of attribute names for attribute selectors instead of checking every element.
- Check only the root element for selectors ending in `:root`.
- Skip selectors whose classes, IDs, or tags are missing from the document anywhere in the selector, not only in its leftmost part.
- Escape text by scanning bytes instead of decoding characters.

### Fixed

- `!important` not recognised when written in uppercase or with whitespace or comments after `!`.
- Only the last element with a duplicated `id` being styled in documents of 1 KB or more.
- Uppercase or mixed-case type selectors (e.g. `H1`) not matching HTML elements in documents of 1 KB or more.
- Class selectors with non-breaking spaces or other non-CSS whitespace in `class` not matching in documents of 1 KB or more.
- Selector lists being split on commas inside `:not()`, attribute values, or strings, so a part of a selector could match on its own.
- Attributes from repeated `<html>` or `<body>` tags being dropped, and existing ones duplicated.
- Selectors starting with a class, ID, or tag that contains escapes or non-ASCII characters not matching in documents of 1 KB or more.
- At-rules written twice when `keep_at_rules` is combined with `remove_inlined_selectors` or `keep_style_tags`.
- Attribute selectors on `class` (`[class]`, `[class*=...]`, etc.) never matching.

## [0.23.0] - 2026-10-16

### Added

- Support for CSS cascade layers (`@layer`). [#340](https://github.com/Stranger6667/css-inline/issues/340)
- Resolving CSS variables (`var()`) to their values with the `resolveCssVariables` option. [#180](https://github.com/Stranger6667/css-inline/issues/180)

### Changed

- Update `reqwest` to `0.13`.

## [0.22.1] - 2026-10-14

### Fixed

- `remove_inlined_selectors` removed `@`-rules from the `<style>` blocks it rewrites when `keep_at_rules` is enabled (it should keep them). [#815](https://github.com/Stranger6667/css-inline/issues/815)

## [0.22.0] - 2026-10-02

### Changed

- Update `cssparser` to `0.38`.
- Update `html5ever` to `0.40`.
- Update `selectors` to `0.41`.

### Fixed

- `remove_inlined_selectors` removed `@`-rules from the `<style>` blocks it rewrites (it should remove only inlined selectors).

### Security

- `class` attribute values not HTML-escaped in the output, allowing markup injection via `"` in a class value. [GHSA-jh54-c7gc-mvcc](https://github.com/Stranger6667/css-inline/security/advisories/GHSA-jh54-c7gc-mvcc)

## [0.21.2] - 2026-08-24

### Changed

- Update `jni` to `0.22`.

### Fixed

- `file://` stylesheet URLs not percent-decoded and broken on Windows drive letters.

## [0.21.1] - 2026-08-08

### Fixed

- With `keepAtRules` enabled, at-rules without a block (e.g. `@import`) lost their prelude and terminator, fusing with the following rule (`@import@media ...`) and producing a stylesheet browsers reject entirely.

## [0.21.0] - 2026-06-16

### Changed

- Update `selectors` to `0.38`.
- Update `cssparser` to `0.37`.

### Fixed

- `width`/`height` HTML attributes not derived from an element's own inline `style` attribute. [#725](https://github.com/Stranger6667/css-inline/issues/725)

## [0.20.2] - 2026-04-02

### Added

- `CssInline.inline(String html, String css)` convenience overload for applying a CSS string to a full HTML document. [#693](https://github.com/Stranger6667/css-inline/issues/693)

### Fixed

- `inlineFragment` silently returning only whitespace when the input starts with whitespace or contains structural HTML tags (`<html>`, `<head>`, `<body>`, `<style>`). [#692](https://github.com/Stranger6667/css-inline/issues/692) [#693](https://github.com/Stranger6667/css-inline/issues/693)

## [0.20.1] - 2026-03-26

### Changed

- Update `html5ever` to `0.39`.
- Update `selectors` to `0.36`.

### Fixed

- `!important` lost when inlining styles onto elements with existing inline `style` attributes. [#682](https://github.com/Stranger6667/css-inline/issues/682)
- `!important` lost when `minify_css` is enabled due to separator mismatch during property lookup. [#682](https://github.com/Stranger6667/css-inline/issues/682)

## [0.20.0] - 2026-02-07

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

### Added

- `removeInlinedSelectors` option to remove selectors that were successfully inlined from `<style>` blocks.

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

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/java-v0.23.0...HEAD
[0.23.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.22.1...java-v0.23.0
[0.22.1]: https://github.com/Stranger6667/css-inline/compare/java-v0.22.0...java-v0.22.1
[0.22.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.21.2...java-v0.22.0
[0.21.2]: https://github.com/Stranger6667/css-inline/compare/java-v0.21.1...java-v0.21.2
[0.21.1]: https://github.com/Stranger6667/css-inline/compare/java-v0.21.0...java-v0.21.1
[0.21.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.20.2...java-v0.21.0
[0.20.2]: https://github.com/Stranger6667/css-inline/compare/java-v0.20.1...java-v0.20.2
[0.20.1]: https://github.com/Stranger6667/css-inline/compare/java-v0.20.0...java-v0.20.1
[0.20.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.19.1...java-v0.20.0
[0.19.1]: https://github.com/Stranger6667/css-inline/compare/java-v0.19.0...java-v0.19.1
[0.19.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.18.0...java-v0.19.0
[0.18.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.17.0...java-v0.18.0
[0.17.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.16.0...java-v0.17.0
[0.16.0]: https://github.com/Stranger6667/css-inline/compare/java-v0.15.0...java-v0.16.0
