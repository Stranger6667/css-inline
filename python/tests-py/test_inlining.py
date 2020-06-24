import pytest

import css_inline


def make_html(style: str, body: str) -> str:
    return "<html><head><title>Test</title><style>{style}</style></head><body>{body}</body></html>".format(
        style=style, body=body
    )


SAMPLE_STYLE = """h1, h2 { color:red; }
strong { text-decoration:none }
p { font-size:2px }
p.footer { font-size: 1px}"""
SAMPLE_INLINED = """<h1 style="color:red;">Big Text</h1>
<p style="font-size:2px ;"><strong style="text-decoration:none ;">Yes!</strong></p>
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
        ({}, make_html(SAMPLE_STYLE, SAMPLE_INLINED)),
        (
            {"remove_style_tags": True},
            "<html><head><title>Test</title></head><body>{body}</body></html>".format(body=SAMPLE_INLINED),
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
