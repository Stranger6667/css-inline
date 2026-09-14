# frozen_string_literal: true

module CSSInline
  module Rails
    module CSSLoaders
      # Both pipelines precompile into `public/` + the assets prefix, so in
      # production the digested file is already on disk and no pipeline object
      # is needed:
      # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/railtie.rb#L42
      module FileSystemLoader
        extend self

        def load(url)
          path = file_name(url)
          ::File.read(path) if path && ::File.file?(path)
        end

        private

        # The prefix is kept here — it is part of the path under `public/` —
        # unlike a mount point, which is not.
        def file_name(url)
          path = Paths.decoded_path(url)
          return if path.nil?

          root = Config.relative_url_root
          path = path.sub(/\A#{Regexp.escape(root.chomp("/"))}/, "") if root && !root.empty?
          contained(::File.join(public_root, path))
        end

        # `..` in an href must not read outside `public/`.
        def contained(path)
          root = ::File.expand_path(public_root)
          expanded = ::File.expand_path(path)
          expanded if expanded.start_with?("#{root}#{::File::SEPARATOR}")
        end

        def public_root
          root = Config.root
          root ? ::File.join(root, "public") : "public"
        end
      end
    end
  end
end
