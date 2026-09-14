# frozen_string_literal: true

module CSSInline
  module Rails
    class Hook
      SKIP_HEADER = :skip_css_inline
      FALSEY = ["", "false", "0"].freeze

      attr_reader :message

      class << self
        # The ActionMailer interceptor interface: `delivering_email` for
        # `register_interceptor`, `previewing_email` for the preview variant.
        def delivering_email(message)
          new(message).perform
          message
        end

        alias_method :previewing_email, :delivering_email
      end

      def initialize(message)
        @message = message
      end

      def perform
        skip_inlining = skip?
        message.header[SKIP_HEADER] = nil
        return if skip_inlining

        inline_html_part if html_part
      end

      private

      # `Mail::Header#[]=` only deletes on `nil`, so `skip_css_inline: false`
      # still creates the field — with an empty value.
      def skip?
        field = message.header[SKIP_HEADER]
        return false if field.nil?

        !FALSEY.include?(field.value.to_s.strip.downcase)
      end

      # Not `content_type.include?`: `multipart/related; type="text/html"`
      # contains "text/html" without being it, and `decoded` raises on it.
      def pure_html_message?
        message.mime_type == "text/html"
      end

      def html_part
        return @html_part if defined?(@html_part)

        @html_part = pure_html_message? ? message : message.html_part
      end

      def inline_html_part
        html = CSSInline::Rails.inline(html_part.decoded)
        html_part.body = html
        html_part.content_type = "text/html; charset=#{html.encoding}"
      end
    end
  end
end
