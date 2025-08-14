<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use CssInline\CSSInliner;

class CssInlineTest extends TestCase
{
    public function testBasicInline(): void
    {
        $html = '<style>h1 { color: blue; }</style><h1>Hello</h1>';
        $result = CssInline\inline($html);
        $this->assertStringContainsString('style="color: blue;"', $result);
        $this->assertStringNotContainsString('<style>', $result);
    }

    public function testInlineFragment(): void
    {
        $html = '<h1>Hello</h1>';
        $css = 'h1 { color: red; }';
        $result = CssInline\inline_fragment($html, $css);
        $this->assertStringContainsString('style="color: red;"', $result);
    }

    public function testCSSInlinerWithExtraCss(): void
    {
        $inliner = new CSSInliner(
            extra_css: 'p { color: green; }'
        );

        $html = '<p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('style="color: green;"', $result);
    }

    public function testCSSInlinerWithDisabledInlining(): void
    {
        $inliner = new CSSInliner(
          inline_style_tags: false,
          keep_style_tags: true,
        );

        $html = '<style>h1 { color: blue; }</style><h1>Test</h1>';
        $result = $inliner->inline($html);

        $this->assertStringNotContainsString('style="color: blue;"', $result);
        $this->assertStringContainsString('<style>', $result);
    }

    public function testCSSInlinerWithBaseUrl(): void
    {
        $inliner = new CSSInliner(
            base_url: 'https://example.com/'
        );

        $html = '<p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertIsString($result);
    }

    public function testMultipleStylesInline(): void
    {
        $html = <<<HTML
        <style>
            h1 { color: blue; font-size: 20px; }
            p { margin: 10px; }
        </style>
        <h1>Title</h1>
        <p>Paragraph</p>
        HTML;

        $result = CssInline\inline($html);

        $this->assertStringContainsString('color: blue', $result);
        $this->assertStringContainsString('font-size: 20px', $result);
        $this->assertStringContainsString('margin: 10px', $result);
    }

    public function testPreserveExistingInlineStyles(): void
    {
        $html = '<style>h1 { color: blue; }</style><h1 style="font-size: 24px;">Hello</h1>';
        $result = CssInline\inline($html);

        // Should merge styles
        $this->assertStringContainsString('color: blue', $result);
        $this->assertStringContainsString('font-size: 24px', $result);
    }
}
