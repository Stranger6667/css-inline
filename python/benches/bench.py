import multiprocessing

import inlinestyler.utils
import premailer
import pynliner
import pytest

import css_inline

SIMPLE_HTML = """<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"""
SIMPLE_HTMLS = [SIMPLE_HTML] * 5000
MERGE_HTML = """<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1 style="background-color: black;">Big Text</h1>
    <p style="background-color: black;">
        <strong style="background-color: black;">Solid</strong>
    </p>
    <p class="footer" style="background-color: black;">Foot notes</p>
</body>
</html>"""
MERGE_HTMLS = [MERGE_HTML] * 5000


def parametrize_functions(*funcs):
    return pytest.mark.parametrize("func", funcs, ids=["css_inline", "premailer", "pynliner", "inlinestyler"])


all_functions = parametrize_functions(
    css_inline.inline, premailer.transform, pynliner.fromString, inlinestyler.utils.inline_css
)


def parallel(func):
    return lambda data: multiprocessing.Pool().map(func, data)


all_many_functions = parametrize_functions(
    css_inline.inline_many,
    parallel(premailer.transform),
    parallel(pynliner.fromString),
    parallel(inlinestyler.utils.inline_css),
)


@all_functions
@pytest.mark.benchmark(group="simple")
def test_simple(benchmark, func):
    benchmark(func, SIMPLE_HTML)


@all_many_functions
@pytest.mark.benchmark(group="simple many")
def test_simple_many(benchmark, func):
    benchmark(func, SIMPLE_HTMLS)


@all_functions
@pytest.mark.benchmark(group="merge")
def test_merge(benchmark, func):
    benchmark(func, MERGE_HTML)


@all_many_functions
@pytest.mark.benchmark(group="merge many")
def test_merge_many(benchmark, func):
    benchmark(func, MERGE_HTMLS)
