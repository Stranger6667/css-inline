# frozen_string_literal: true

module CSSInline
  module Rails
    module CSSLoaders
      # Mirrors how Propshaft's own middleware serves `/assets/*`:
      # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/server.rb#L16-L20
      module PropshaftLoader
        extend self

        def load(url)
          return unless available?

          asset = find_asset(url)
          assembly.compilers.compile(asset) if asset
        end

        private

        # `extract_path_and_digest` is Propshaft's, so the digest format tracks
        # upstream rather than being copied:
        # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/asset.rb#L11-L16
        #
        # Its pattern also matches an ordinary hyphenated name, so the path is
        # tried as written before the digest is stripped.
        def find_asset(url)
          path = Paths.relative(url)
          return if path.nil?

          undigested, = ::Propshaft::Asset.extract_path_and_digest(path)
          assembly.load_path.find(path) || assembly.load_path.find(undigested)
        end

        def assembly
          Config.application.assets
        end

        # Sprockets also answers `application.assets`, with a different object.
        def available?
          return false unless defined?(::Propshaft)

          app = Config.application
          app.respond_to?(:assets) && app.assets.is_a?(::Propshaft::Assembly)
        end
      end
    end
  end
end
