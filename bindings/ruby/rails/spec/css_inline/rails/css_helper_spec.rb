# frozen_string_literal: true

require "spec_helper"

RSpec.describe CSSInline::Rails::CSSHelper do
  # Writes real stylesheets into a real public/ and points a real application at
  # it, so the default strategy chain resolves them for real.
  def with_stylesheets(files)
    Dir.mktmpdir do |dir|
      FileUtils.mkdir_p(File.join(dir, "public/assets"))
      files.each { |name, css| File.write(File.join(dir, "public/assets", name), css) }
      use_rails_app(root: dir)
      yield
    end
  end

  it "returns nil when the document links no stylesheets" do
    expect(described_class.css_for_html(sample_html)).to be_nil
  end

  it "concatenates every linked stylesheet, in document order" do
    with_stylesheets("a.css" => "h1 { color: red; }", "b.css" => "h1 { font-size: 2px; }") do
      html = <<~HTML
        <html><head>
          <link rel="stylesheet" href="/assets/a.css">
          <link rel="stylesheet" href="/assets/b.css">
        </head><body></body></html>
      HTML

      expect(described_class.css_for_html(html))
        .to eq("h1 { color: red; }\nh1 { font-size: 2px; }")
    end
  end

  it "skips a link marked data-css-inline=ignore" do
    with_stylesheets("a.css" => "h1 { color: red; }", "b.css" => "h1 { font-size: 2px; }") do
      html = <<~HTML
        <html><head>
          <link rel="stylesheet" href="/assets/a.css">
          <link rel="stylesheet" href="/assets/b.css" data-css-inline="ignore">
        </head><body></body></html>
      HTML

      expect(described_class.css_for_html(html)).to eq("h1 { color: red; }")
    end
  end

  it "leaves a remote stylesheet to css_inline instead of failing delivery" do
    CSSInline::Rails.config[:strategies] = []
    html = linked_html("https://fonts.googleapis.com/css2?family=Inter")

    expect(described_class.css_for_html(html)).to be_nil
  end

  # `stylesheet_link_tag` prefixes the asset host, which apps also set at
  # runtime (`ActionMailer::Base.asset_host = ...`), so the path decides.
  ["https://gitlab.example.com", "//cdn.example.com"].each do |host|
    it "resolves #{host}/assets/a.css to the local asset" do
      with_stylesheets("a.css" => "h1 { color: red; }") do
        expect(described_class.css_for_html(linked_html("#{host}/assets/a.css")))
          .to eq("h1 { color: red; }")
      end
    end
  end

  it "skips a protocol-relative href" do
    CSSInline::Rails.config[:strategies] = []

    expect(described_class.css_for_html(linked_html("//cdn.example.com/a.css"))).to be_nil
  end

  it "does not scan past </head>" do
    CSSInline::Rails.config[:strategies] = []
    html = '<html><head></head><body><link rel="stylesheet" href="/assets/a.css"></body></html>'

    expect(described_class.css_for_html(html)).to be_nil
  end

  it "still finds links when the document has no </head>" do
    strategy = RecordingStrategy.new("h1 {}")
    CSSInline::Rails.config[:strategies] = [strategy]

    expect(described_class.css_for_html('<link rel="stylesheet" href="/assets/a.css">')).to eq("h1 {}")
  end

  it "ignores link tags that are not stylesheets" do
    html = '<html><head><link rel="icon" href="/favicon.ico"></head><body></body></html>'

    expect(described_class.css_for_html(html)).to be_nil
  end

  it "falls through to the next strategy when one declines" do
    # FileSystemLoader genuinely declines here: nothing is on disk.
    answering = RecordingStrategy.new("h1 {}")
    CSSInline::Rails.config[:strategies] = [CSSInline::Rails::CSSLoaders::FileSystemLoader, answering]

    expect(described_class.css_for_html(linked_html("/assets/a.css"))).to eq("h1 {}")
    expect(answering.urls).to eq(["/assets/a.css"])
  end

  it "names a mistyped strategy rather than failing on NoMethodError" do
    CSSInline::Rails.config[:strategies] = [:sprocket]

    expect { described_class.css_for_html(linked_html("/assets/a.css")) }
      .to raise_error(ArgumentError, /Unknown css_inline-rails strategy: :sprocket/)
  end

  it "accepts any object implementing the strategy contract" do
    CSSInline::Rails.config[:strategies] = [RecordingStrategy.new("h1 { color: red; }")]

    expect(described_class.css_for_html(linked_html("/assets/a.css"))).to eq("h1 { color: red; }")
  end

  it "raises when no strategy resolves the stylesheet" do
    CSSInline::Rails.config[:strategies] = []

    expect { described_class.css_for_html(linked_html("/assets/a.css")) }
      .to raise_error(described_class::FileNotFound, %r{/assets/a\.css})
  end

  it "tags loaded CSS as UTF-8 whatever encoding the strategy returned" do
    CSSInline::Rails.config[:strategies] = [RecordingStrategy.new('h1:before { content: "\u2014"; }'.b)]

    css = described_class.css_for_html(linked_html("/assets/a.css"))

    expect(css.encoding).to eq(Encoding::UTF_8)
    expect(css).to be_valid_encoding
  end

  describe "caching" do
    it "does not cache without an application" do
      strategy = RecordingStrategy.new("h1 {}")
      CSSInline::Rails.config[:strategies] = [strategy]

      2.times { described_class.css_for_html(linked_html("/assets/a.css")) }

      expect(strategy.urls.size).to eq(2)
    end

    it "does not cache when the application reloads code" do
      use_rails_app(enable_reloading: true)
      strategy = RecordingStrategy.new("h1 {}")
      CSSInline::Rails.config[:strategies] = [strategy]

      2.times { described_class.css_for_html(linked_html("/assets/a.css")) }

      expect(strategy.urls.size).to eq(2)
    end

    # Keyed on the reloading flag, not the environment name, so a `staging`
    # environment caches like production.
    it "loads each URL once when the application does not reload code" do
      use_rails_app(enable_reloading: false)
      strategy = RecordingStrategy.new("h1 {}")
      CSSInline::Rails.config[:strategies] = [strategy]

      2.times { described_class.css_for_html(linked_html("/assets/a.css")) }

      expect(strategy.urls.size).to eq(1)
    end
  end
end
