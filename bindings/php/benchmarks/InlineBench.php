<?php

declare(strict_types=1);

namespace CssInline\Benchmarks;

use Pelago\Emogrifier\CssInliner;
use PhpBench\Benchmark\Metadata\Annotations\ParamProviders;
use TijsVerkoyen\CssToInlineStyles\CssToInlineStyles;

class InlineBench
{
    // Large/complex pages that other libraries can't handle
    private const SKIP_FOR_OTHER_LIBS = ['big_page'];

    private CssToInlineStyles $cssToInlineStyles;

    public function __construct()
    {
        $this->cssToInlineStyles = new CssToInlineStyles();
        ini_set('pcre.backtrack_limit', '10000000');
        ini_set('pcre.recursion_limit', '10000000');
        ini_set('memory_limit', '2048M');
    }

    /**
     * @ParamProviders("provideAllCases")
     */
    public function benchCssInline(array $params): void
    {
        \CssInline\inline($params['html']);
    }

    /**
     * @ParamProviders("provideSmallCases")
     */
    public function benchCssToInlineStyles(array $params): void
    {
        $this->cssToInlineStyles->convert($params['html']);
    }

    /**
     * @ParamProviders("provideSmallCases")
     */
    public function benchEmogrifier(array $params): void
    {
        CssInliner::fromHtml($params['html'])->inlineCss()->render();
    }

    public function provideAllCases(): \Generator
    {
        yield from $this->loadBenchmarks(skipLarge: false);
    }

    public function provideSmallCases(): \Generator
    {
        yield from $this->loadBenchmarks(skipLarge: true);
    }

    private function loadBenchmarks(bool $skipLarge): \Generator
    {
        $jsonPath = __DIR__ . '/../../../benchmarks/benchmarks.json';
        $json = file_get_contents($jsonPath);
        $benchmarks = json_decode($json, true);

        foreach ($benchmarks as $benchmark) {
            if ($skipLarge && in_array($benchmark['name'], self::SKIP_FOR_OTHER_LIBS, true)) {
                continue;
            }
            yield $benchmark['name'] => [
                'html' => $benchmark['html'],
            ];
        }
    }
}
