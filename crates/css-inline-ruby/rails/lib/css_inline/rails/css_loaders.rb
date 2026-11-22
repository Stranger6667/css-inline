# frozen_string_literal: true

require "uri"

module CSSInline
  module Rails
    module CSSLoaders
      # Every read of Rails state goes through here. `::Rails` can exist without
      # an application (rails-html-sanitizer defines the namespace), and
      # `::Rails.configuration` raises until one boots.
      module App
        extend self

        def application
          return unless defined?(::Rails) && ::Rails.respond_to?(:application)

          ::Rails.application
        end

        def config
          application&.config
        end

        def root
          return unless defined?(::Rails) && ::Rails.respond_to?(:root)

          ::Rails.root
        end

        # https://guides.rubyonrails.org/configuring.html#config-relative-url-root
        def relative_url_root
          config.respond_to?(:relative_url_root) ? config.relative_url_root : nil
        end

        # `/assets` by default:
        # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/railtie.rb#L11
        def assets_prefix
          assets = config.respond_to?(:assets) ? config.assets : nil
          assets&.prefix
        end

        # Code reloading means assets can change between messages too.
        def reloading?
          config.nil? || config.enable_reloading
        end

        def url_prefix
          ::File.join(relative_url_root.to_s, assets_prefix.to_s, "/")
        end

        # The logical asset path: `/sub/assets/a.css` -> `a.css`.
        def relative_path(url)
          decoded_path(url)&.delete_prefix(url_prefix)
        end

        # `nil` for anything unusable, so a loader declines instead of raising
        # out of the delivery interceptor.
        def decoded_path(url)
          path = URI.parse(url.to_s).path.to_s
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
