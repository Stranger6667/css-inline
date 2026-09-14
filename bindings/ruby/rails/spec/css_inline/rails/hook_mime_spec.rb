# frozen_string_literal: true

require "spec_helper"

# Message shapes ported from premailer-rails' spec/support/fixtures/message.rb,
# which is the part of that gem with 110M downloads of field testing behind it.
RSpec.describe CSSInline::Rails::Hook do
  HTML_WITH_CSS = <<~HTML
    <html><head><style>p { color: red; }</style></head>
    <body><p>Lorem ipsum</p></body></html>
  HTML

  def base_message
    Mail.new do
      to "some@email.com"
      from "from@example.com"
      subject "testing css_inline-rails"
    end
  end

  def deliver(message)
    described_class.delivering_email(message)
  end

  describe "multipart/mixed with an attachment around a multipart/alternative" do
    subject(:message) do
      message = base_message
      alternative = Mail::Part.new(content_type: "multipart/alternative")
      alternative.text_part = Mail::Part.new do
        content_type "text/plain; charset=UTF-8"
        body "Lorem ipsum"
      end
      alternative.html_part = Mail::Part.new do
        content_type "text/html; charset=UTF-8"
        body HTML_WITH_CSS
      end
      message.add_part(alternative)
      message.add_file(filename: "foo.png", content: "foobar")
      message.ready_to_send!
      message
    end

    it "inlines the html part nested inside the alternative" do
      expect(deliver(message).html_part.decoded).to include('style="color: red;"')
    end

    it "leaves the text part alone" do
      expect(deliver(message).text_part.decoded).to include("Lorem ipsum")
      expect(deliver(message).text_part.decoded).not_to include("style=")
    end

    it "leaves the attachment intact" do
      delivered = deliver(message)

      expect(delivered.attachments.size).to eq(1)
      expect(delivered.attachments.first.filename).to eq("foo.png")
      expect(delivered.attachments.first.decoded).to eq("foobar")
    end
  end

  # `multipart/related; type="text/html"` contains the string "text/html"
  # without being a bare html body. Reading it as one made `decoded` raise
  # inside the interceptor and the message never sent.
  describe "multipart/related with an inline image" do
    subject(:message) do
      message = base_message
      message.content_type 'multipart/related; type="text/html"; start="<html-root@example.com>"'
      message.add_part(
        Mail::Part.new do
          content_type "text/html; charset=UTF-8"
          content_id "<html-root@example.com>"
          content_location "/index.html"
          body HTML_WITH_CSS
        end
      )
      message.add_file(filename: "logo.png", content: "img")
      message.ready_to_send!
      message
    end

    it "delivers rather than raising" do
      expect { deliver(message) }.not_to raise_error
    end

    it "inlines the html part" do
      expect(deliver(message).html_part.decoded).to include('style="color: red;"')
    end

    it "preserves the html part headers" do
      delivered = deliver(message)

      expect(delivered.content_type_parameters["start"]).to eq("<html-root@example.com>")
      expect(delivered.html_part.content_id).to eq("<html-root@example.com>")
      expect(delivered.html_part.content_location).to eq("/index.html")
    end
  end

  describe "a text-only message" do
    subject(:message) do
      message = base_message
      message.body "Lorem ipsum"
      message.content_type "text/plain; charset=UTF-8"
      message.ready_to_send!
      message
    end

    it "is left untouched" do
      expect(deliver(message).decoded).to eq("Lorem ipsum")
    end
  end

  describe "character encodings" do
    def html_message_with(body, charset)
      message = base_message
      message.body body
      message.content_type "text/html; charset=#{charset}"
      message.ready_to_send!
      message
    end

    it "keeps non-latin characters" do
      body = HTML_WITH_CSS.sub("Lorem ipsum", "٩(-̮̮̃-̃)۶ 🎉")

      expect(deliver(html_message_with(body, "UTF-8")).decoded).to include("🎉")
    end

    it "keeps typographic dashes" do
      body = HTML_WITH_CSS.sub("Lorem ipsum", "a—b – c")

      expect(deliver(html_message_with(body, "UTF-8")).decoded).to include("a—b – c")
    end

    it "transcodes an ISO-8859-7 body and relabels the charset" do
      body = HTML_WITH_CSS.sub("Lorem ipsum", "Αα Ββ Γγ").encode(Encoding::ISO_8859_7)
      delivered = deliver(html_message_with(body, "ISO-8859-7"))

      # css_inline always returns UTF-8, so the part must not keep claiming 8859-7.
      expect(delivered.content_type).to match(/charset=utf-8/i)
      expect(delivered.decoded.force_encoding("UTF-8")).to include("Αα Ββ Γγ")
    end
  end
end
