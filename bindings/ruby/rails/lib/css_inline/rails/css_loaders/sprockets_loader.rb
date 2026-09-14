# frozen_string_literal: true

module CSSInline
  module Rails
    module CSSLoaders
      # `assets_manifest` is what sprockets-rails hangs off the application:
      # https://github.com/rails/sprockets-rails/blob/v3.5.2/lib/sprockets/railtie.rb#L34
      #
      # `find_sources` returns the compiled source for a logical path:
      # https://github.com/rails/sprockets/blob/v4.4.1/lib/sprockets/manifest.rb#L139-L152
      module SprocketsLoader
        extend self

        # Sprockets appends an MD5 (32) or SHA256 (64) hex digest before the
        # extension. Unlike Propshaft it exposes no helper to strip one.
        DIGEST = /-(\h{32}|\h{64})\.css\z/

        def load(url)
          return unless available?

          path = Paths.relative(url)
          return if path.nil?

          manifest.find_sources(path).first || manifest.find_sources(path.sub(DIGEST, ".css")).first
        rescue Errno::ENOENT, TypeError
          nil
        end

        private

        def manifest
          Config.application.assets_manifest
        end

        def available?
          app = Config.application
          app.respond_to?(:assets_manifest) && app.assets_manifest
        end
      end
    end
  end
end
