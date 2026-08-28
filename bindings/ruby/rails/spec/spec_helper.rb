# frozen_string_literal: true

# Before the gem, so the Railtie path is exercised.
require "rails"
require "action_mailer"
require "css_inline/rails"

require_relative "support/rails_application"
require_relative "support/pipelines"

RSpec.configure do |config|
  config.disable_monkey_patching!
  config.order = :random
  Kernel.srand config.seed

  config.expect_with(:rspec) { |c| c.syntax = :expect }

  # `double`/`allow`/`receive` are undefined, so no stub can creep back in.
  config.mock_with :nothing

  config.before do
    CSSInline::Rails.config = CSSInline::Rails.default_config
    CSSInline::Rails::CSSHelper.cache = {}
  end
end

SAMPLE_HTML = <<~HTML
  <html>
    <head><style>h1 { color: red; }</style></head>
    <body><h1>Hello</h1></body>
  </html>
HTML

def linked_html(href)
  <<~HTML
    <html>
      <head><link rel="stylesheet" href="#{href}"></head>
      <body><h1>Hello</h1></body>
    </html>
  HTML
end

LINKED_HTML = linked_html("/assets/email-0123456789abcdef.css")

def html_message(body: SAMPLE_HTML)
  Mail.new do
    to "to@example.com"
    from "from@example.com"
    content_type "text/html; charset=UTF-8"
    body body
  end
end

# Caches `@html_part`; a list swap alone is not enough.
def multipart_message(html: SAMPLE_HTML)
  Mail.new do
    to "to@example.com"
    from "from@example.com"
    text_part { body "Hello" }
    html_part do
      content_type "text/html; charset=UTF-8"
      body html
    end
  end
end

# How ActionMailer assembles one: no cached part.
def multipart_message_via_add_part(html: SAMPLE_HTML)
  message = Mail.new do
    to "to@example.com"
    from "from@example.com"
  end
  message.add_part(Mail::Part.new { content_type "text/plain"; body "Hello" })
  message.add_part(
    Mail::Part.new do
      content_type "text/html; charset=UTF-8"
      body html
    end
  )
  message
end
