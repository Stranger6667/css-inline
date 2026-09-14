# frozen_string_literal: true

require "nokogiri"
require "uri"

module CSSInline
  module Rails
    # `css_inline` raises on a `<link>` href it cannot fetch unless
    # `load_remote_stylesheets` is off, and it cannot reach an `/assets/...`
    # path at all — only the asset pipeline can. So the hrefs are resolved here
    # and handed over as `extra_css`.
    module CSSHelper
      extend self

      HEAD_END = %r{</head\s*>}i

      FileNotFound = Class.new(StandardError)

      attr_accessor :cache
      self.cache = {}

      def css_for_html(html)
        urls = stylesheet_urls(html)
        return if urls.empty?

        urls.map { |url| css_for_url(url) }.join("\n")
      end

      def css_for_url(url)
        return cache[url] ||= load_css(url) if cache_enabled?

        load_css(url)
      end

      # Remote hrefs are left to `css_inline`, which drops them silently rather
      # than failing a delivery over a CDN font sheet.
      def local_stylesheet?(url)
        uri = URI.parse(url)
        uri.scheme.nil? && uri.host.nil?
      rescue URI::InvalidURIError
        false
      end

      private

      # Only `<head>` is parsed, and only to read hrefs — the original string is
      # what gets inlined. Parsing a whole message costs more than the inlining
      # itself; parsing just the head is ~1000x cheaper on a large document.
      def stylesheet_urls(html)
        Nokogiri::HTML(head_of(html))
          .css('link[rel="stylesheet"]')
          .reject { |link| link["data-css-inline"] == "ignore" }
          .map { |link| link["href"].to_s }
          .reject(&:empty?)
          .select { |url| local_stylesheet?(url) }
      end

      def head_of(html)
        match = HEAD_END.match(html)
        match ? html[0, match.end(0)] : html
      end

      def load_css(url)
        CSSInline::Rails.config.fetch(:strategies).each do |strategy|
          css = strategy_for(strategy).load(url)
          return css if css
        end

        raise FileNotFound, %(Stylesheet "#{url}" could not be loaded by any strategy.)
      end

      # A non-symbol is a strategy object of its own; an unknown symbol is a typo.
      def strategy_for(key)
        case key
        when :filesystem then CSSLoaders::FileSystemLoader
        when :sprockets then CSSLoaders::SprocketsLoader
        when :propshaft then CSSLoaders::PropshaftLoader
        when Symbol then raise ArgumentError, "Unknown css_inline-rails strategy: #{key.inspect}"
        else key
        end
      end

      # Anything but development and test, so `staging` does not recompile every
      # stylesheet for every message.
      def cache_enabled?
        env = CSSLoaders::Config.env
        return false if env.nil?

        !(env.development? || env.test?)
      end
    end
  end
end
