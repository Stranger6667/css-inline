# frozen_string_literal: true

require "concurrent/map"
require "nokogiri"
require "uri"

module CSSInline
  module Rails
    # `css_inline` cannot fetch an `/assets/...` href; only the asset pipeline
    # can resolve one. So the hrefs are resolved here and handed over as
    # `extra_css`.
    module CSSHelper
      extend self

      HEAD_END = %r{</head\s*>}i

      class FileNotFound < Error; end

      attr_accessor :cache
      self.cache = Concurrent::Map.new

      def css_for_html(html)
        css = stylesheet_urls(html).filter_map { |url| css_for_url(url) }
        css.join("\n") unless css.empty?
      end

      def css_for_url(url)
        return load_css(url) if CSSLoaders::App.reloading?

        cache.compute_if_absent(url) { load_css(url) }
      end

      private

      # Only `<head>` is parsed, and only to read hrefs: the links live there
      # and the body can be large. `css_inline` gets the original string.
      def stylesheet_urls(html)
        Nokogiri::HTML(head_of(html))
          .css('link[rel="stylesheet"]')
          .reject { |link| link["data-css-inline"] == "ignore" }
          .map { |link| link["href"].to_s }
          .reject(&:empty?)
          .select { |url| web_url?(url) }
      end

      # `data:` and `cid:` hrefs are not files any strategy can find.
      def web_url?(url)
        scheme = URI.parse(url).scheme
        scheme.nil? || %w[http https].include?(scheme)
      rescue URI::InvalidURIError
        false
      end

      def head_of(html)
        match = HEAD_END.match(html)
        match ? html[0, match.end(0)] : html
      end

      # Sprockets reads precompiled files as binary and a custom strategy can
      # return anything, so the result is relabelled as UTF-8.
      #
      # Like premailer-rails, the path decides, so a link on the asset host
      # resolves locally. An unresolved link with a host is someone else's,
      # such as a font CDN, and is skipped.
      def load_css(url)
        CSSInline::Rails.config.fetch(:strategies).each do |strategy|
          css = strategy_for(strategy).load(url)
          return String.new(css, encoding: Encoding::UTF_8) if css
        end
        return if URI.parse(url).host

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
    end
  end
end
