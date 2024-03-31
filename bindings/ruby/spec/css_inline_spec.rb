# frozen_string_literal: true

require 'rspec'
require_relative "../lib/css_inline"

def make_html(style, body)
  "<html><head><style>#{style}</style></head><body>#{body}</body></html>"
end

SAMPLE_STYLE = """
h1, h2 { color:red; }
strong { text-decoration:none }
p { font-size:2px }
p.footer { font-size: 1px}
"""

SAMPLE_INLINED = "<h1 style=\"color: red;\">Big Text</h1><p style=\"font-size: 2px;\"><strong style=\"text-decoration: none;\">Yes!</strong></p><p class=\"footer\" style=\"font-size: 1px;\">Foot notes</p>"
SAMPLE_HTML = make_html(
    SAMPLE_STYLE,
    "<h1>Big Text</h1><p><strong>Yes!</strong></p><p class=\"footer\">Foot notes</p>"
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
EXPECTED_INLINED_FRAGMENT = "<main>\n<h1 style=\"color: blue;\">Hello</h1>\n<section>\n<p style=\"color: red;\">who am i</p>\n</section>\n</main>"

funcs = [
  ["CSSInline::inline", ->(html, **kwargs){ CSSInline::inline(html, **kwargs) }],
  ["CSSInline::inline_many", ->(html, **kwargs){ CSSInline::inline_many([html, html], **kwargs) }],
  ["CSSInline::CSSInliner::inline", ->(html, **kwargs){ CSSInline::CSSInliner.new(**kwargs).inline(html) }],
  ["CSSInline::CSSInliner::inline_many", ->(html, **kwargs){ CSSInline::CSSInliner.new(**kwargs).inline_many([html, html]) }],
]

test_cases = [
  ["keep style tags", { keep_style_tags: true }, make_html(SAMPLE_STYLE, SAMPLE_INLINED)],
  ["drop style tags", {}, "<html><head></head><body>#{SAMPLE_INLINED}</body></html>"]
]

RSpec.describe 'CssInline' do
  funcs.each do |(fname, func)|
    test_cases.each do |(name, kwargs, expected)|
      it "Inline CSS - #{fname} - #{name}" do
        result = func.call(SAMPLE_HTML, **kwargs)
        result = result[0] if result.is_a? Array
        expect(result).to eq(expected)
      end
    end
  end

  it 'Uses optional keyword arguments for configuration' do
    inliner = CSSInline::CSSInliner.new(keep_style_tags: true)
    inlined = inliner.inline(SAMPLE_HTML)
    expected = make_html(
      SAMPLE_STYLE,
      SAMPLE_INLINED
    )
    expect(inlined).to eq(expected)
  end

  it 'Inlines multiple documents in parallel' do
    inliner = CSSInline::CSSInliner.new(keep_style_tags: true)
    inlined = inliner.inline_many([SAMPLE_HTML, SAMPLE_HTML])
    expected = make_html(
      SAMPLE_STYLE,
      SAMPLE_INLINED
    )
    expect(inlined[0]).to eq(expected)
    expect(inlined[1]).to eq(expected)
  end

  it 'Inlines CSS into HTML fragments' do
    inlined = CSSInline::inline_fragment(FRAGMENT, CSS)
    expect(inlined).to eq(EXPECTED_INLINED_FRAGMENT)
  end

  it 'Inlines CSS into HTML fragments via method' do
    inliner = CSSInline::CSSInliner.new()
    inlined = inliner.inline_fragment(FRAGMENT, CSS)
    expect(inlined).to eq(EXPECTED_INLINED_FRAGMENT)
  end

  it 'Inlines CSS into multiple HTML fragments in parallel' do
    inlined = CSSInline::inline_many_fragments([FRAGMENT, FRAGMENT], [CSS, CSS])
    expect(inlined[0]).to eq(EXPECTED_INLINED_FRAGMENT)
    expect(inlined[1]).to eq(EXPECTED_INLINED_FRAGMENT)
  end

  it 'Inlines CSS into multiple HTML fragments in parallel via method' do
    inliner = CSSInline::CSSInliner.new()
    inlined = inliner.inline_many_fragments([FRAGMENT, FRAGMENT], [CSS, CSS])
    expect(inlined[0]).to eq(EXPECTED_INLINED_FRAGMENT)
    expect(inlined[1]).to eq(EXPECTED_INLINED_FRAGMENT)
  end

  [
    ["http://127.0.0.1:1234/external.css", {}],
    ["../../css-inline/tests/external.css", {}],
    ["external.css", {"base_url": "http://127.0.0.1:1234"}]
  ].each do |href, kwargs|
      it "Resolves remote stylesheets: #{href}" do
        # Fails with the following on Windows:
        # error trying to connect: tcp set_nonblocking error: An operation was attempted on something that is not a socket. (os error 10038)
        skip 'Skipping on Windows' if RUBY_PLATFORM =~ /mswin|mingw|cygwin/
        inlined = CSSInline::inline("<html>
<head>
<link href=\"#{href}\" rel=\"stylesheet\">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>", **kwargs)
        expect(inlined).to eq('''<html><head>


</head>
<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>''')
      end
  end

  it 'Caches remote stylesheets' do
    inliner = CSSInline::CSSInliner.new(cache: CSSInline::StylesheetCache.new(size: 5))
    expect(inliner.inline(SAMPLE_HTML)).to eq("<html><head></head><body>#{SAMPLE_INLINED}</body></html>")
  end

  it 'Caches remote stylesheets with default' do
    inliner = CSSInline::CSSInliner.new(cache: CSSInline::StylesheetCache.new())
    expect(inliner.inline(SAMPLE_HTML)).to eq("<html><head></head><body>#{SAMPLE_INLINED}</body></html>")
  end

  [
    0,
    -1,
    "foo"
  ].each do |size|
    it "Errors on invalid cache size - #{size}" do
      expect { CSSInline::CSSInliner.new(cache: CSSInline::StylesheetCache.new(size: size)) }.to raise_error('Cache size must be an integer greater than zero')
    end
  end

  it 'Shows the stylesheet location in base url errors' do
    expect { CSSInline::CSSInliner.new(base_url: 'foo') }.to raise_error('relative URL without a base: foo')
  end

  it 'Shows the stylesheet location in network errors' do
    expect {
        CSSInline::CSSInliner.new.inline('''
        <html>
        <head>
        <link href="http:" rel="stylesheet">
        </head>
        <body>
        </body>
        </html>
        ''')
    }.to raise_error('builder error: http:')
  end

  it 'Shows the CSS parsing errors' do
    expect {
        CSSInline::CSSInliner.new.inline('''
        <html>
    <head>
    </head>
    <style>h1, h2 { color:red; }</style>
    <body>
    <h1 style="@wrong { color: --- }">Hello world!</h1>
    </body>
    </html>
        ''')
    }.to raise_error('Invalid @ rule: wrong')
  end

end
