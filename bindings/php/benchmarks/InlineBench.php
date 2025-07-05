<?php

namespace CssInline\Benchmarks;

use PhpBench\Benchmark\Metadata\Annotations\ParamProviders;
use CssInline;
use TijsVerkoyen\CssToInlineStyles\CssToInlineStyles;

ini_set('pcre.backtrack_limit', 1000000);

class InlineBench
{
    private CssToInlineStyles $cssToInlineStyles;

    public function __construct()
    {
        $this->cssToInlineStyles = new CssToInlineStyles();
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
