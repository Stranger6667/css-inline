# frozen_string_literal: true

require "spec_helper"

RSpec.describe CSSInline::Rails::Hook do
  it "inlines the body of a pure text/html message" do
    message = described_class.delivering_email(html_message)

    expect(message.decoded).to include('style="color: red;"')
    expect(message.content_type).to include("text/html")
  end

  # The two ways a multipart message gets built differ: the block form caches
  # `@html_part`, the `add_part` form (what ActionMailer uses) does not.
  [:multipart_message, :multipart_message_via_add_part].each do |builder|
    context "a multipart message built with #{builder}" do
      it "inlines the html part" do
        message = described_class.delivering_email(send(builder))

        expect(message.html_part.decoded).to include('style="color: red;"')
      end

      it "leaves the text part alone" do
        message = described_class.delivering_email(send(builder))

        expect(message.text_part.decoded).to eq("Hello")
      end

      it "replaces the html part instead of appending a second one" do
        message = described_class.delivering_email(send(builder))

        expect(message.parts.size).to eq(2)
        html_parts = message.parts.select { |p| p.content_type.to_s.include?("text/html") }
        expect(html_parts.size).to eq(1)
      end
    end
  end

  it "does nothing to a message with no html part" do
    message = Mail.new do
      to "to@example.com"
      from "from@example.com"
      body "just text"
    end

    expect { described_class.delivering_email(message) }.not_to raise_error
    expect(message.decoded).to eq("just text")
  end

  describe "the skip header" do
    it "leaves the message uninlined" do
      message = html_message
      message.header[:skip_css_inline] = true

      described_class.delivering_email(message)

      expect(message.decoded).not_to include("style=")
    end

    it "is stripped so it never reaches the recipient" do
      message = html_message
      message.header[:skip_css_inline] = true

      described_class.delivering_email(message)

      expect(message.header[:skip_css_inline]).to be_nil
    end

    [false, "", "0"].each do |value|
      it "is stripped without skipping inlining when set to #{value.inspect}" do
        message = html_message
        message.header[:skip_css_inline] = value

        described_class.delivering_email(message)

        expect(message.header[:skip_css_inline]).to be_nil
        expect(message.decoded).to include('style="color: red;"')
      end
    end
  end

  it "resolves a linked stylesheet through the configured strategies" do
    CSSInline::Rails.config[:strategies] = [RecordingStrategy.new("h1 { color: blue; }")]

    message = described_class.delivering_email(html_message(body: LINKED_HTML))

    expect(message.decoded).to include('style="color: blue;"')
  end

  # css_inline raises ArgumentError on an href it cannot resolve, so an
  # unresolvable link must not reach it.
  it "raises FileNotFound when no strategy resolves a linked stylesheet" do
    CSSInline::Rails.config[:strategies] = []

    expect { described_class.delivering_email(html_message(body: LINKED_HTML)) }
      .to raise_error(CSSInline::Rails::CSSHelper::FileNotFound, /email-0123456789abcdef\.css/)
  end

  it "ignores a link marked data-css-inline=ignore" do
    CSSInline::Rails.config[:strategies] = []
    html = LINKED_HTML.sub("<link ", '<link data-css-inline="ignore" ')

    expect { described_class.delivering_email(html_message(body: html)) }.not_to raise_error
  end
end
