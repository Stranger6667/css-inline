<?php

declare(strict_types=1);

namespace CssInline;

/**
 * Library version.
 */
const VERSION = '0.19.0';

/**
 * Inline CSS from a style tag into matching elements.
 */
function inline(string $html): string {}

/**
 * Inline CSS into an HTML fragment.
 */
function inlineFragment(string $fragment, string $css): string {}

/**
 * Inline CSS from style tags into matching elements for multiple HTML documents.
 *
 * @param string[] $htmls
 * @return string[]
 */
function inlineMany(array $htmls): array {}

/**
 * Inline CSS into multiple HTML fragments.
 *
 * @param string[] $htmls
 * @return string[]
 */
function inlineManyFragments(array $htmls, string $css): array {}

/**
 * Exception thrown when CSS inlining fails.
 */
class InlineError extends \Exception {}

/**
 * Cache for external stylesheets.
 */
class StylesheetCache
{
    /**
     * @param int $size Maximum number of stylesheets to cache
     */
    public function __construct(int $size) {}
}

/**
 * High-performance CSS inliner.
 */
class CssInliner
{
    /**
     * @param bool $inlineStyleTags Whether to inline CSS from "style" tags
     * @param bool $keepStyleTags Keep "style" tags after inlining
     * @param bool $keepLinkTags Keep "link" tags after inlining
     * @param bool $keepAtRules Keep "at-rules" (e.g., @media) after inlining
     * @param bool $minifyCss Remove trailing semicolons and spaces
     * @param bool $loadRemoteStylesheets Whether to load remote stylesheets
     * @param string|null $baseUrl Base URL for resolving relative URLs
     * @param string|null $extraCss Additional CSS to inline
     * @param int $preallocateNodeCapacity Pre-allocate capacity for HTML nodes
     * @param StylesheetCache|null $cache Cache for external stylesheets
     * @param bool $removeInlinedSelectors Remove selectors that were successfully inlined from style blocks
     */
    public function __construct(
        bool $inlineStyleTags = true,
        bool $keepStyleTags = false,
        bool $keepLinkTags = false,
        bool $keepAtRules = false,
        bool $minifyCss = false,
        bool $loadRemoteStylesheets = true,
        ?string $baseUrl = null,
        ?string $extraCss = null,
        int $preallocateNodeCapacity = 32,
        ?StylesheetCache $cache = null,
        bool $removeInlinedSelectors = false,
    ) {}

    /**
     * Inline CSS from style tags into matching elements.
     */
    public function inline(string $html): string {}

    /**
     * Inline CSS into an HTML fragment.
     */
    public function inlineFragment(string $html, string $css): string {}

    /**
     * Inline CSS from style tags into matching elements for multiple HTML documents.
     *
     * @param string[] $htmls
     * @return string[]
     */
    public function inlineMany(array $htmls): array {}

    /**
     * Inline CSS into multiple HTML fragments.
     *
     * @param string[] $htmls
     * @return string[]
     */
    public function inlineManyFragments(array $htmls, string $css): array {}
}
