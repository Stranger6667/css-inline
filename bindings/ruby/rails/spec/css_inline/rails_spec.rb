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

  it "reads config with method syntax, as `config.css_inline` in an application" do
    expect(described_class.config.strategies).to eq(%i[filesystem sprockets propshaft])
  end

  # Apps that order interceptors themselves call this, as with premailer-rails.
  it "registers the delivery interceptor on demand" do
    described_class.register_interceptors
    message = html_message
    message.delivery_method(:test)

    message.deliver

    expect(Mail::TestMailer.deliveries.last.decoded).to include('style="color: red;"')
  end

  it "roots its errors in one class" do
    expect(CSSInline::Rails::CSSHelper::FileNotFound.ancestors).to include(CSSInline::Rails::Error)
  end

  describe "gemspec" do
    subject(:spec) { Gem::Specification.load(File.expand_path("../../css_inline-rails.gemspec", __dir__)) }

    def requirement(name)
      spec.dependencies.find { |dep| dep.name == name }.requirement.to_s
    end

    it "pins css_inline to its own minor version" do
      minor = CSSInline::Rails::VERSION.split(".").first(2).join(".")

      expect(requirement("css_inline")).to eq("~> #{minor}.0")
    end

    it "requires the oldest Rails that CI tests" do
      expect(requirement("actionmailer")).to eq(">= 7.1")
    end
  end

  describe ".inline" do
    # premailer keeps them too; dropping them breaks responsive and dark-mode mail.
    it "keeps media queries by default" do
      html = sample_html.sub("</style>", "@media (max-width: 600px) { h1 { color: blue; } }</style>")

      expect(described_class.inline(html)).to include("@media (max-width: 600px)")
    end

    it "drops media queries with keep_at_rules: false" do
      described_class.config[:inline_options] = {keep_at_rules: false}
      html = sample_html.sub("</style>", "@media (max-width: 600px) { h1 { color: blue; } }</style>")

      expect(described_class.inline(html)).not_to include("@media")
    end

    it "passes inline_options through to css_inline" do
      described_class.config[:inline_options] = {keep_style_tags: true}

      expect(described_class.inline(sample_html)).to include("<style>")
    end

    it "combines configured extra_css with resolved stylesheets" do
      described_class.config[:inline_options] = {extra_css: "h1 { font-size: 2px; }"}
      described_class.config[:strategies] = [RecordingStrategy.new("h1 { color: blue; }")]

      result = described_class.inline(linked_html("/assets/a.css"))

      expect(result).to include("font-size: 2px;")
      expect(result).to include("color: blue;")
    end

    # Left on, css_inline raises on an /assets/... href it cannot fetch.
    it "forces load_remote_stylesheets off even if configured on" do
      described_class.config[:inline_options] = {load_remote_stylesheets: true}
      described_class.config[:strategies] = []

      html = linked_html("/assets/missing.css").sub("<link ", '<link data-css-inline="ignore" ')

      expect { described_class.inline(html) }.not_to raise_error
    end

    it "combines non-ASCII extra_css with a stylesheet read under a non-UTF-8 locale" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/a.css"), %(h1 { font-family: "\u2014"; }))
        use_rails_app(root: dir)
        described_class.config[:inline_options] = {extra_css: %(body { font-family: "\u00e9"; })}
        original = Encoding.default_external
        Encoding.default_external = Encoding::US_ASCII

        expect(described_class.inline(linked_html("/assets/a.css"))).to include("\u2014", "\u00e9")
      ensure
        Encoding.default_external = original
      end
    end
  end
end
