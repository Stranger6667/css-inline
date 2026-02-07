<?php

declare(strict_types=1);

use CssInline\CssInliner;
use CssInline\StylesheetCache;
use PHPUnit\Framework\TestCase;

class CssInlineTest extends TestCase
{
    public function testVersion(): void
    {
        $this->assertMatchesRegularExpression('/^\d+\.\d+\.\d+$/', \CssInline\VERSION);
    }

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
        $result = CssInline\inlineFragment($html, $css);
        $this->assertStringContainsString('style="color: red;"', $result);
    }

    public function testInlineMany(): void
    {
        $html1 = '<style>h1 { color: blue; }</style><h1>Hello</h1>';
        $html2 = '<style>p { color: red; }</style><p>World</p>';
        $results = CssInline\inlineMany([$html1, $html2]);

        $this->assertCount(2, $results);
        $this->assertStringContainsString('style="color: blue;"', $results[0]);
        $this->assertStringContainsString('style="color: red;"', $results[1]);
    }

    public function testInlineManyFragments(): void
    {
        $html1 = '<h1>Hello</h1>';
        $html2 = '<h1>World</h1>';
        $css = 'h1 { color: green; }';
        $results = CssInline\inlineManyFragments([$html1, $html2], $css);

        $this->assertCount(2, $results);
        $this->assertStringContainsString('style="color: green;"', $results[0]);
        $this->assertStringContainsString('style="color: green;"', $results[1]);
    }

    public function testCssInlinerWithExtraCss(): void
    {
        $inliner = new CssInliner(
            extraCss: 'p { color: green; }'
        );

        $html = '<p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('style="color: green;"', $result);
    }

    public function testCssInlinerWithDisabledInlining(): void
    {
        $inliner = new CssInliner(
            inlineStyleTags: false,
            keepStyleTags: true,
        );

        $html = '<style>h1 { color: blue; }</style><h1>Test</h1>';
        $result = $inliner->inline($html);

        $this->assertStringNotContainsString('style="color: blue;"', $result);
        $this->assertStringContainsString('<style>', $result);
    }

    public function testCssInlinerWithBaseUrl(): void
    {
        $inliner = new CssInliner(
            baseUrl: 'https://example.com/'
        );

        $html = '<p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertIsString($result);
    }

    public function testCssInlinerInlineFragment(): void
    {
        $inliner = new CssInliner();
        $html = '<h1>Hello</h1>';
        $css = 'h1 { color: purple; }';
        $result = $inliner->inlineFragment($html, $css);

        $this->assertStringContainsString('style="color: purple;"', $result);
    }

    public function testCssInlinerInlineMany(): void
    {
        $inliner = new CssInliner();
        $html1 = '<style>h1 { color: blue; }</style><h1>Hello</h1>';
        $html2 = '<style>p { color: red; }</style><p>World</p>';
        $results = $inliner->inlineMany([$html1, $html2]);

        $this->assertCount(2, $results);
        $this->assertStringContainsString('style="color: blue;"', $results[0]);
        $this->assertStringContainsString('style="color: red;"', $results[1]);
    }

    public function testCssInlinerInlineManyFragments(): void
    {
        $inliner = new CssInliner();
        $html1 = '<h1>Hello</h1>';
        $html2 = '<h1>World</h1>';
        $css = 'h1 { color: orange; }';
        $results = $inliner->inlineManyFragments([$html1, $html2], $css);

        $this->assertCount(2, $results);
        $this->assertStringContainsString('style="color: orange;"', $results[0]);
        $this->assertStringContainsString('style="color: orange;"', $results[1]);
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

        $this->assertStringContainsString('color: blue', $result);
        $this->assertStringContainsString('font-size: 24px', $result);
    }

    public function testKeepAtRules(): void
    {
        $inliner = new CssInliner(
            keepAtRules: true,
        );

        $html = '<style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style><h1>Test</h1>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('style="color: blue;"', $result);
        $this->assertStringContainsString('@media', $result);
    }

    public function testMinifyCss(): void
    {
        $inliner = new CssInliner(
            minifyCss: true,
        );

        $html = '<style>h1 { color: blue; font-weight: bold; }</style><h1>Test</h1>';
        $result = $inliner->inline($html);

        // Minified CSS should not have trailing semicolon or extra spaces
        $this->assertStringContainsString('style="color:blue;font-weight:bold"', $result);
    }

    public function testStylesheetCache(): void
    {
        $cache = new StylesheetCache(size: 10);
        $inliner = new CssInliner(
            cache: $cache,
        );

        $html = '<p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertIsString($result);
    }

    public function testKeepLinkTags(): void
    {
        $inliner = new CssInliner(
            keepLinkTags: true,
            loadRemoteStylesheets: false,
        );

        $html = '<link rel="stylesheet" href="style.css"><p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('<link', $result);
    }

    public function testDataCssInlineIgnore(): void
    {
        $html = '<style>h1 { color: blue; }</style><h1 data-css-inline="ignore">Hello</h1>';
        $result = CssInline\inline($html);

        $this->assertStringNotContainsString('style="color: blue;"', $result);
        $this->assertStringContainsString('data-css-inline="ignore"', $result);
    }

    public function testDataCssInlineKeep(): void
    {
        $inliner = new CssInliner(
            keepStyleTags: false,
        );

        $html = '<style data-css-inline="keep">h1 { color: blue; }</style><h1>Hello</h1>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('<style', $result);
        $this->assertStringContainsString('style="color: blue;"', $result);
    }

    public function testInvalidBaseUrl(): void
    {
        $this->expectException(\Exception::class);

        new CssInliner(
            baseUrl: 'not-a-valid-url'
        );
    }

    public function testInvalidCacheSize(): void
    {
        $this->expectException(\Exception::class);

        new StylesheetCache(size: 0);
    }

    public function testNegativePreallocateNodeCapacity(): void
    {
        $this->expectException(\Exception::class);

        new CssInliner(
            preallocateNodeCapacity: -1
        );
    }

    public function testPreallocateNodeCapacity(): void
    {
        $inliner = new CssInliner(
            preallocateNodeCapacity: 100
        );

        $html = '<style>h1 { color: blue; }</style><h1>Hello</h1>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('style="color: blue;"', $result);
    }

    public function testLoadRemoteStylesheetsDisabled(): void
    {
        $inliner = new CssInliner(
            loadRemoteStylesheets: false,
        );

        // When remote stylesheets are disabled, link tags should be ignored
        $html = '<link rel="stylesheet" href="https://example.com/style.css"><p>Test</p>';
        $result = $inliner->inline($html);

        $this->assertIsString($result);
    }

    public function testApplyWidthAttributes(): void
    {
        $inliner = new CssInliner(
            applyWidthAttributes: true,
        );

        $html = '<html><head><style>td { width: 100px; }</style></head><body><table><tr><td>Test</td></tr></table></body></html>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('width="100"', $result);
    }

    public function testApplyHeightAttributes(): void
    {
        $inliner = new CssInliner(
            applyHeightAttributes: true,
        );

        $html = '<html><head><style>td { height: 50px; }</style></head><body><table><tr><td>Test</td></tr></table></body></html>';
        $result = $inliner->inline($html);

        $this->assertStringContainsString('height="50"', $result);
    }

    public function testRemoveInlinedSelectors(): void
    {
        $inliner = new CssInliner(
            removeInlinedSelectors: true,
        );

        $html = '<html><head><style>h1 { color: blue; } h2 { color: red; }</style></head><body><h1>Test</h1></body></html>';
        $result = $inliner->inline($html);

        $this->assertEquals(
            '<html><head><style>h2 { color: red; }</style></head><body><h1 style="color: blue;">Test</h1></body></html>',
            $result
        );
    }
}
