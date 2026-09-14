# frozen_string_literal: true

require "spec_helper"

RSpec.describe CSSInline::Rails do
  describe ".default_config" do
    it "hands out an independent copy each time" do
      first = described_class.default_config
      first[:strategies] << :extra

      expect(described_class.default_config[:strategies]).not_to include(:extra)
    end
  end

  describe ".inline" do
    it "passes inline_options through to css_inline" do
      described_class.config[:inline_options] = { keep_style_tags: true }

      expect(described_class.inline(SAMPLE_HTML)).to include("<style>")
    end

    it "combines configured extra_css with resolved stylesheets" do
      described_class.config[:inline_options] = { extra_css: "h1 { font-size: 2px; }" }
      described_class.config[:strategies] = [RecordingStrategy.new("h1 { color: blue; }")]

      result = described_class.inline(linked_html("/assets/a.css"))

      expect(result).to include("font-size: 2px;")
      expect(result).to include("color: blue;")
    end

    # Left on, css_inline raises on an /assets/... href it cannot fetch.
    it "forces load_remote_stylesheets off even if configured on" do
      described_class.config[:inline_options] = { load_remote_stylesheets: true }
      described_class.config[:strategies] = []

      html = linked_html("/assets/missing.css").sub("<link ", '<link data-css-inline="ignore" ')

      expect { described_class.inline(html) }.not_to raise_error
    end
  end
end
