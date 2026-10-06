import json
import multiprocessing
import pathlib

# inlinestyler is excluded: it crashes with current lxml ('CSSSelector' object has no attribute 'evaluate')
import premailer
import pynliner
import pytest
import toronado

import css_inline

HERE = pathlib.Path(__file__).parent

with (HERE.parents[2] / "benchmarks/benchmarks.json").open() as f:
    benchmark_data = json.load(f)


def parametrize_functions(*funcs, ids=("css_inline", "premailer", "pynliner", "toronado")):
    return pytest.mark.parametrize("func", funcs, ids=ids)


all_functions = parametrize_functions(
    css_inline.inline,
    premailer.transform,
    pynliner.fromString,
    toronado.from_string,
)


def parallel(func):
    return lambda data: multiprocessing.Pool().map(func, data)


all_many_functions = parametrize_functions(
    css_inline.inline_many,
    parallel(css_inline.inline),
    parallel(premailer.transform),
    parallel(pynliner.fromString),
    parallel(toronado.from_string),
    ids=(
        "css_inline",
        "css_inline_pyprocess",
        "premailer",
        "pynliner",
        "toronado",
    ),
)


for benchmark in benchmark_data:
    name = benchmark["name"]
    html = benchmark["html"]
    if len(html) < 1000:
        repeat = 2000
    else:
        repeat = 200
    htmls = [html] * repeat

    exec(
        f"""
@all_functions
@pytest.mark.benchmark(group="{name}")
def test_benchmark_{name}(benchmark, func):
    benchmark(func, '''{html}''')
    """
    )

    exec(
        f"""
@all_many_functions
@pytest.mark.benchmark(group="{name} many")
def test_benchmark_{name}_many(benchmark, func):
    benchmark(func, {htmls})
    """
    )

    # register the functions into the module namespace
    globals()[f"test_benchmark_{name}"] = locals()[f"test_benchmark_{name}"]
    globals()[f"test_benchmark_{name}_many"] = locals()[f"test_benchmark_{name}_many"]
