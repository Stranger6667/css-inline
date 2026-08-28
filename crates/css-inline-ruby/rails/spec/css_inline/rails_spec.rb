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

  def run_ruby(script)
    output, status = Open3.capture2e(RbConfig.ruby, "-e", script)
    raise output unless status.success?

    output
  end

  # Railties and an asset pipeline are already loaded in this process.
  it "inlines with ActionMailer alone" do
    output = run_ruby(<<~RUBY)
      require "action_mailer"
      require "css_inline/rails"
      abort "railties loaded" if defined?(::Rails::Railtie)
      abort "propshaft loaded" if defined?(::Propshaft)
      CSSInline::Rails.register_interceptors
      stylesheet = Object.new
      def stylesheet.load(_url) = "h1 { color: red; }"
      CSSInline::Rails.config.strategies += [stylesheet]
      mail = Mail.new(from: "a@example.com", to: "b@example.com", content_type: "text/html",
        body: '<link rel="stylesheet" href="/assets/email.css"><h1>Hello</h1>')
      mail.delivery_method(:test)
      mail.deliver
      print Mail::TestMailer.deliveries.last.decoded
    RUBY

    expect(output).to include('style="color: red;"')
  end

  describe "in a booted application" do
    # A process holds one application, hence the subprocess.
    def deliver_from_fresh_app(app_config: "", require_after_boot: false)
      require_gem = 'require "css_inline/rails"'
      script = <<~RUBY
        require "rails"
        require "action_mailer/railtie"
        #{require_gem unless require_after_boot}
        class FreshApp < Rails::Application
          config.eager_load = false
          config.logger = Logger.new(IO::NULL)
          config.secret_key_base = "0" * 64
          config.action_mailer.delivery_method = :test
          #{app_config}
        end
        FreshApp.initialize!
        #{require_gem if require_after_boot}
        class FreshMailer < ActionMailer::Base
          def hello
            mail(from: "a@example.com", to: "b@example.com", content_type: "text/html",
              body: "<style>h1 { color: red; }</style><h1>Hello</h1>")
          end
        end
        FreshMailer.hello.deliver_now
        print ActionMailer::Base.deliveries.last.decoded
      RUBY
      run_ruby(script)
    end

    # `require: false` plus a require in config/initializers.
    it "installs the hook when required after the application booted" do
      expect(deliver_from_fresh_app(require_after_boot: true)).to include('style="color: red;"')
    end

    # As with `config.css_inline = config_for(:css_inline)`.
    it "reads a config.css_inline the application assigns anew" do
      output = deliver_from_fresh_app(app_config: "config.css_inline = {inline_options: {keep_style_tags: true}}")

      expect(output).to include("<style>", 'style="color: red;"')
    end
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

    it "inlines linked stylesheets with inline_style_tags: false" do
      described_class.config[:inline_options] = {inline_style_tags: false}
      described_class.config[:strategies] = [RecordingStrategy.new("h1 { color: blue; }")]

      expect(described_class.inline(linked_html("/assets/a.css"))).to include('<h1 style="color: blue;">')
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

    # Dart Sass' compressed output starts with one when the CSS has non-ASCII.
    it "applies the first rule of a stylesheet that starts with a BOM" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/a.css"), %(\uFEFFh1{color:red}))
        use_rails_app(root: dir)

        expect(described_class.inline(linked_html("/assets/a.css"))).to include('<h1 style="color: red;">')
      end
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
