# frozen_string_literal: true

module CSSInline
  module Rails
    module CSSLoaders
      # Mirrors how Propshaft's own middleware serves `/assets/*`:
      # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/server.rb#L16-L20
      module PropshaftLoader
        extend self

        # Propshaft's own pattern, the same in 0.x and 1.x. Only 1.x exposes it,
        # as `Propshaft::Asset.extract_path_and_digest`:
        # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/asset.rb#L11-L16
        DIGEST = /-([0-9a-zA-Z]{7,128})\.(?!digested)([^.]|.map)+\z/

        def load(url)
          return unless available?

          asset = find_asset(url)
          assembly.compilers.compile(asset) if asset
        end

        private

        # The pattern also matches an ordinary hyphenated name, so the path is
        # tried as written before the digest is stripped.
        def find_asset(url)
          path = App.relative_path(url)
          return if path.nil?

          digest = path[DIGEST, 1]
          undigested = digest ? path.sub("-#{digest}", "") : path
          assembly.load_path.find(path) || assembly.load_path.find(undigested)
        end

        def assembly
          App.application.assets
        end

        # Sprockets also answers `application.assets`, with a different object.
        def available?
          return false unless defined?(::Propshaft)

          app = App.application
          app.respond_to?(:assets) && app.assets.is_a?(::Propshaft::Assembly)
        end
      end
    end
  end
end
