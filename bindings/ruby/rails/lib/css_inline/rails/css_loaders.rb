# frozen_string_literal: true

require "uri"

module CSSInline
  module Rails
    module CSSLoaders
      # Every read of a Rails global goes through here. Railties can be loaded
      # without a booted application, and `::Rails.configuration` is
      # `::Rails.application.config`, which raises in that state.
      module Config
        extend self

        def application
          return unless defined?(::Rails) && ::Rails.respond_to?(:application)

          ::Rails.application
        end

        def config
          application&.config
        end

        def env
          return unless defined?(::Rails) && ::Rails.respond_to?(:env)

          ::Rails.env
        end

        def root
          return unless defined?(::Rails) && ::Rails.respond_to?(:root)

          ::Rails.root
        end

        # https://guides.rubyonrails.org/configuring.html#config-relative-url-root
        def relative_url_root
          config.respond_to?(:relative_url_root) ? config.relative_url_root : nil
        end

        # The URL prefix assets are served under, `/assets` by default:
        # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/railtie.rb#L11
        def assets_prefix
          assets = config.respond_to?(:assets) ? config.assets : nil
          assets&.prefix
        end
      end

      module Prefix
        extend self

        def build
          ::File.join(Config.relative_url_root.to_s, Config.assets_prefix.to_s, "/")
        end
      end

      module Paths
        extend self

        def relative(url)
          path = decoded_path(url)
          return if path.nil?

          path.sub(/\A#{Regexp.escape(Prefix.build)}/, "")
        end

        # `nil` for anything unusable, so a loader declines instead of raising
        # out of the delivery interceptor.
        def decoded_path(url)
          uri = URI.parse(url.to_s)
          path = uri.path.to_s
          return if path.empty?

          URI.decode_www_form_component(path)
        rescue URI::InvalidURIError, ArgumentError
          nil
        end
      end
    end
  end
end

require "css_inline/rails/css_loaders/file_system_loader"
require "css_inline/rails/css_loaders/sprockets_loader"
require "css_inline/rails/css_loaders/propshaft_loader"
