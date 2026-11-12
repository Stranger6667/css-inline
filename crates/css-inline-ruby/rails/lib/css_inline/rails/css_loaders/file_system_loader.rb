# frozen_string_literal: true

module CSSInline
  module Rails
    module CSSLoaders
      # Both pipelines precompile into `public/` + the assets prefix, so in
      # production the digested file is already on disk:
      # https://github.com/rails/propshaft/blob/v1.3.2/lib/propshaft/railtie.rb#L42
      module FileSystemLoader
        extend self

        # UTF-8 regardless of locale: under `LANG=C` Rails' `default_internal`
        # makes a plain read raise on the first non-ASCII byte.
        def load(url)
          path = file_name(url)
          ::File.read(path, encoding: Encoding::UTF_8) if path && ::File.file?(path)
        end

        private

        # The assets prefix stays, it is a directory under `public/`. The
        # relative URL root goes, it is a mount point.
        def file_name(url)
          path = App.decoded_path(url)
          return if path.nil?

          root = App.relative_url_root
          path = path.delete_prefix(root.chomp("/")) if root && !root.empty?
          contained(::File.join(public_root, path))
        end

        # `..` in an href must not read outside `public/`.
        def contained(path)
          root = ::File.expand_path(public_root)
          expanded = ::File.expand_path(path)
          expanded if expanded.start_with?("#{root}#{::File::SEPARATOR}")
        end

        def public_root
          root = App.root
          root ? ::File.join(root, "public") : "public"
        end
      end
    end
  end
end
