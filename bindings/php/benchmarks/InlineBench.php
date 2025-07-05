<?php

namespace CssInline\Benchmarks;

use PhpBench\Benchmark\Metadata\Annotations\ParamProviders;
use CssInline;
use TijsVerkoyen\CssToInlineStyles\CssToInlineStyles;
use Pelago\Emogrifier\CssInliner;

class InlineBench
{
    private CssToInlineStyles $cssToInlineStyles;

    public function __construct()
    {
        $this->cssToInlineStyles = new CssToInlineStyles();
        ini_set('pcre.backtrack_limit', '10000000');
        ini_set('pcre.recursion_limit', '10000000');
        ini_set('memory_limit', '2048M');
    }

    /**
     * @ParamProviders("provideBenchmarkCases")
     */
    public function benchCssInline(array $params): void
    {
        \CssInline\inline($params['html']);
    }

    /**
     * @ParamProviders("provideBenchmarkCases")
     */
    public function benchCssToInlineStyles(array $params): void
    {
        $this->cssToInlineStyles->convert($params['html']);
    }

    /**
     * @ParamProviders("provideBenchmarkCases")
     */
    public function benchEmogrifier(array $params): void
    {
        CssInliner::fromHtml($params['html'])->inlineCss()->render();
    }


    public function provideBenchmarkCases(): \Generator
    {
        $jsonPath = __DIR__ . '/../../../benchmarks/benchmarks.json';
        $json = file_get_contents($jsonPath);
        $benchmarks = json_decode($json, true);

        foreach ($benchmarks as $benchmark) {
            yield $benchmark['name'] => [
                'html' => $benchmark['html']
            ];
        }
    }
}
