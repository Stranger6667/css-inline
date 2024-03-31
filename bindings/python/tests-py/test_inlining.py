from contextlib import suppress

import pytest
from hypothesis import given, provisional, settings
from hypothesis import strategies as st

import css_inline


def make_html(style: str, body: str) -> str:
    return "<html><head><style>{style}</style></head><body>{body}</body></html>".format(
        style=style, body=body
    )


SAMPLE_STYLE = """h1, h2 { color:red; }
strong { text-decoration:none }
p { font-size:2px }
p.footer { font-size: 1px}"""
SAMPLE_INLINED = """<h1 style="color: red;">Big Text</h1>
<p style="font-size: 2px;"><strong style="text-decoration: none;">Yes!</strong></p>
<p class="footer" style="font-size: 1px;">Foot notes</p>"""


@pytest.mark.parametrize(
    "func",
    (
        lambda html, **kwargs: css_inline.inline(html, **kwargs),
        lambda html, **kwargs: css_inline.inline_many([html], **kwargs),
        lambda html, **kwargs: css_inline.CSSInliner(**kwargs).inline(html),
        lambda html, **kwargs: css_inline.CSSInliner(**kwargs).inline_many([html]),
    ),
)
@pytest.mark.parametrize(
    "kwargs, expected",
    (
        ({"keep_style_tags": True}, make_html(SAMPLE_STYLE, SAMPLE_INLINED)),
        (
            {"keep_style_tags": False},
            "<html><head></head><body>{body}</body></html>".format(body=SAMPLE_INLINED),
        ),
    ),
)
def test_no_existing_style(func, kwargs, expected):
    html = make_html(
        SAMPLE_STYLE,
        """<h1>Big Text</h1>
<p><strong>Yes!</strong></p>
<p class="footer">Foot notes</p>""",
    )
    result = func(html, **kwargs)
    if isinstance(result, list):
        result = result[0]
    assert result == expected


def test_inline_many_wrong_type():
    with pytest.raises(TypeError):
        css_inline.inline_many([1])


def test_missing_stylesheet():
    with pytest.raises(
        css_inline.InlineError, match="Missing stylesheet file: tests/missing.css"
    ):
        css_inline.inline(
            """<html>
<head>
<link href="tests/missing.css" rel="stylesheet">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"""
        )


def test_file_scheme():
    css_inline.inline(
        """<html>
<head>
<link href="external.css" rel="stylesheet">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>""",
        base_url="file://tests-py/",
    )


@pytest.mark.parametrize(
    "href, kwargs",
    (
        ("http://127.0.0.1:1234/external.css", {}),
        ("../../css-inline/tests/external.css", {}),
        ("external.css", {"base_url": "http://127.0.0.1:1234"}),
    ),
)
def test_remote_stylesheet(href, kwargs):
    inlined = css_inline.inline(
        f"""<html>
<head>
<link href="{href}" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 {{ color: red; }}
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>""",
        **kwargs,
    )
    assert (
        inlined
        == """<html><head>

<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">

</head>
<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"""
    )


def test_invalid_base_url():
    with pytest.raises(ValueError, match="relative URL without a base: foo"):
        css_inline.CSSInliner(base_url="foo")


def test_invalid_href():
    with pytest.raises(ValueError, match="builder error: http:"):
        css_inline.inline(
            """<html>
    <head>
    <link href="http:" rel="stylesheet">
    </head>
    <body>
    </body>
    </html>"""
        )


def test_invalid_style():
    with pytest.raises(ValueError, match="Invalid @ rule: wrong"):
        css_inline.inline(
            """<html>
    <head>
    </head>
    <style>h1, h2 { color:red; }</style>
    <body>
    <h1 style="@wrong { color: --- }">Hello world!</h1>
    </body>
    </html>"""
        )


def test_cache():
    inliner = css_inline.CSSInliner(cache=css_inline.StylesheetCache(size=3))
    html = """<html>
<head>
<link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"""
    assert (
        inliner.inline(html)
        == """<html><head>

</head>
<body>
<h1 style="color: blue;">Big Text</h1>

</body></html>"""
    )


FRAGMENT = """<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>"""
CSS = """
p {
    color: red;
}

h1 {
    color: blue;
}
"""
EXPECTED_INLINED_FRAGMENT = '<main>\n<h1 style="color: blue;">Hello</h1>\n<section>\n<p style="color: red;">who am i</p>\n</section>\n</main>'


def test_inline_fragment():
    assert css_inline.inline_fragment(FRAGMENT, CSS) == EXPECTED_INLINED_FRAGMENT


def test_inline_fragment_method():
    assert (
        css_inline.CSSInliner().inline_fragment(FRAGMENT, CSS)
        == EXPECTED_INLINED_FRAGMENT
    )


def test_inline_many_fragments():
    assert css_inline.inline_many_fragments([FRAGMENT, FRAGMENT], [CSS, CSS]) == [
        EXPECTED_INLINED_FRAGMENT,
        EXPECTED_INLINED_FRAGMENT,
    ]


def test_inline_many_fragments_method():
    assert css_inline.CSSInliner().inline_many_fragments(
        [FRAGMENT, FRAGMENT], [CSS, CSS]
    ) == [
        EXPECTED_INLINED_FRAGMENT,
        EXPECTED_INLINED_FRAGMENT,
    ]


@pytest.mark.parametrize("size", (0, -1, "foo"))
def test_invalid_cache(size):
    with pytest.raises(
        ValueError, match="Cache size must be an integer greater than zero"
    ):
        css_inline.StylesheetCache(size=size)


@given(
    document=st.text(),
    keep_style_tags=st.booleans() | st.none(),
    base_url=provisional.urls() | st.none(),
    load_remote_stylesheets=st.booleans() | st.none(),
    extra_css=st.text() | st.none(),
)
@settings(max_examples=1000)
def test_random_input(
    document,
    keep_style_tags,
    base_url,
    load_remote_stylesheets,
    extra_css,
):
    with suppress(ValueError):
        inliner = css_inline.CSSInliner(
            keep_style_tags=keep_style_tags,
            base_url=base_url,
            load_remote_stylesheets=load_remote_stylesheets,
            extra_css=extra_css,
        )
        inliner.inline(document)
