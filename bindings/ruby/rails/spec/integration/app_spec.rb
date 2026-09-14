# frozen_string_literal: true

# Boots a real Rails application and delivers a real message through it. This is
# the only spec that proves the Railtie registers the interceptor and that asset
# resolution works against a live pipeline.
#
# Rails allows one application per process, so this file runs on its own (see
# the Rakefile) and picks its pipeline from PIPELINE=propshaft|sprockets.

require "rails"
require "action_mailer/railtie"

PIPELINE = ENV.fetch("PIPELINE", "propshaft")

case PIPELINE
when "propshaft" then require "propshaft"
when "sprockets" then require "sprockets/railtie"
else raise ArgumentError, "Unknown PIPELINE: #{PIPELINE}"
end

require "css_inline/rails"
require "rspec"

# Same guarantee as the unit specs: this file drives a real application, and
# `double`/`allow` are not available to undermine that.
RSpec.configure { |config| config.mock_with :nothing }

class TestApp < ::Rails::Application
  config.root = File.expand_path("fixtures", __dir__)
  config.eager_load = false
  config.consider_all_requests_local = true
  config.secret_key_base = "0" * 64
  config.logger = Logger.new(IO::NULL)
  config.action_mailer.delivery_method = :test
  config.action_mailer.default_url_options = { host: "example.com" }
  config.assets.prefix = "/assets"
end

TestApp.initialize!

class TestMailer < ActionMailer::Base
  default from: "from@example.com"

  def hello
    mail(to: "to@example.com", subject: "Hello")
  end

  def skipped
    mail(to: "to@example.com", subject: "Skipped", skip_css_inline: true) do |format|
      format.html { render "hello" }
    end
  end
end

RSpec.describe "a booted Rails application (#{PIPELINE})" do
  before { ActionMailer::Base.deliveries.clear }

  it "registers the interceptor without any configuration" do
    expect(Mail.class_variable_get(:@@delivery_interceptors)).to include(CSSInline::Rails::Hook)
  end

  it "registers a preview interceptor too" do
    expect(ActionMailer::Base.preview_interceptors).to include(CSSInline::Rails::Hook)
  end

  it "inlines a stylesheet linked with stylesheet_link_tag" do
    TestMailer.hello.deliver_now

    expect(ActionMailer::Base.deliveries.last.decoded).to include('style="color: rebeccapurple;"')
  end

  it "removes the link tag it inlined" do
    TestMailer.hello.deliver_now

    expect(ActionMailer::Base.deliveries.last.decoded).not_to include("<link")
  end

  it "resolves the digested asset URL the view actually rendered" do
    # Guards the digest-stripping regexes against a real pipeline's naming.
    html = TestMailer.hello.html_part&.decoded || TestMailer.hello.body.decoded

    expect(html).to match(%r{/assets/email-\w+\.css})
  end

  it "leaves a message alone when told to skip" do
    TestMailer.skipped.deliver_now

    expect(ActionMailer::Base.deliveries.last.decoded).not_to include("style=")
  end

  it "does not leak the skip header to the recipient" do
    TestMailer.skipped.deliver_now

    expect(ActionMailer::Base.deliveries.last.header[:skip_css_inline]).to be_nil
  end
end
