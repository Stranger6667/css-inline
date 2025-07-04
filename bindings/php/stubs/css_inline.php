<?php

// Stubs for css_inline

namespace CssInline {
    function inline(string $html): string {}

    function inline_fragment(string $fragment, string $css): string {}

    class InlineError extends \Exception {
        public function __construct() {}
    }

    class CssInliner {
        public function inline(string $html): string {}

        public function inlineFragment(string $html, string $css): string {}

        public function __construct(?bool $inline_style_tags, ?bool $keep_style_tags, ?bool $keep_link_tags, ?bool $load_remote_stylesheets, ?string $base_url, ?string $extra_css) {}
    }
}
