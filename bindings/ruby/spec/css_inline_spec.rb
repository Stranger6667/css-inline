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

  it 'Shows the stylesheet location in base url errors' do
    expect { CSSInline::CSSInliner.new(base_url: 'foo') }.to raise_error('relative URL without a base: foo')
  end

  it 'Shows the stylesheet location in network errors' do
    expect {
        CSSInline::CSSInliner.new.inline('''
        <html>
        <head>
        <link href="http:" rel="stylesheet" type="text/css">
        </head>
        <body>
        </body>
        </html>
        ''')
    }.to raise_error('builder error: empty host: http:')
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
