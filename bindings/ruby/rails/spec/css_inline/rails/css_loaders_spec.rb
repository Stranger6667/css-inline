# frozen_string_literal: true

require "spec_helper"

RSpec.describe CSSInline::Rails::CSSLoaders do
  describe CSSInline::Rails::CSSLoaders::PropshaftLoader do
    it "resolves a digested URL through a real Propshaft assembly" do
      with_propshaft do |assembly, url|
        use_rails_app(assets: assembly)

        expect(described_class.load(url)).to eq(Pipelines::CSS)
      end
    end

    it "resolves an undigested URL" do
      with_propshaft do |assembly, _url|
        use_rails_app(assets: assembly)

        expect(described_class.load("/assets/email.css")).to eq(Pipelines::CSS)
      end
    end

    it "honours a non-default assets prefix" do
      with_propshaft(prefix: "/static") do |assembly, url|
        use_rails_app(assets: assembly, prefix: "/static")

        expect(described_class.load(url)).to eq(Pipelines::CSS)
      end
    end

    it "honours a relative_url_root" do
      with_propshaft do |assembly, url|
        use_rails_app(assets: assembly, relative_url_root: "/sub")

        expect(described_class.load(File.join("/sub", url))).to eq(Pipelines::CSS)
      end
    end

    it "runs the asset through Propshaft's compilers rather than reading the file" do
      # CssAssetUrls rewrites url(...) references to digested paths, so compiled
      # output must differ from the source on disk.
      css = "h1 { background: url(email.css); }\n"
      with_propshaft(css: css) do |assembly, url|
        use_rails_app(assets: assembly)

        expect(described_class.load(url)).to match(%r{url\("?/assets/email-\w+\.css"?\)})
      end
    end

    it "resolves a hyphenated name the digest pattern also matches" do
      with_propshaft do |assembly, _url|
        use_rails_app(assets: assembly)
        # "email-newsletter.css" looks exactly like a digested "email.css".
        assembly.load_path.find("email.css")

        expect(described_class.load("/assets/email.css")).to eq(Pipelines::CSS)
      end
    end

    it "declines on a malformed href instead of raising" do
      with_propshaft do |assembly, _url|
        use_rails_app(assets: assembly)

        expect { described_class.load("/assets/a b{}.css") }.not_to raise_error
      end
    end

    it "returns nil for an asset that is not in the load path" do
      with_propshaft do |assembly, _url|
        use_rails_app(assets: assembly)

        expect(described_class.load("/assets/missing.css")).to be_nil
      end
    end

    it "returns nil when the application has no Propshaft assembly" do
      use_rails_app(assets: nil)

      expect(described_class.load("/assets/email.css")).to be_nil
    end
  end

  describe CSSInline::Rails::CSSLoaders::SprocketsLoader do
    it "resolves a digested URL through a real Sprockets manifest" do
      with_sprockets do |manifest, url|
        use_rails_app(assets_manifest: manifest)

        expect(described_class.load(url)).to eq(Pipelines::CSS)
      end
    end

    it "resolves an undigested URL" do
      with_sprockets do |manifest, _url|
        use_rails_app(assets_manifest: manifest)

        expect(described_class.load("/assets/email.css")).to eq(Pipelines::CSS)
      end
    end

    it "honours a non-default assets prefix" do
      with_sprockets(prefix: "/static") do |manifest, url|
        use_rails_app(assets_manifest: manifest, prefix: "/static")

        expect(described_class.load(url)).to eq(Pipelines::CSS)
      end
    end

    it "returns nil for an asset the manifest does not know" do
      with_sprockets do |manifest, _url|
        use_rails_app(assets_manifest: manifest)

        expect(described_class.load("/assets/missing.css")).to be_nil
      end
    end

    it "returns nil when the application has no Sprockets manifest" do
      use_rails_app(assets_manifest: nil)

      expect(described_class.load("/assets/email.css")).to be_nil
    end
  end

  describe CSSInline::Rails::CSSLoaders::FileSystemLoader do
    it "reads a precompiled asset out of public/" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/email-abc.css"), Pipelines::CSS)
        use_rails_app(root: dir)

        expect(described_class.load("/assets/email-abc.css")).to eq(Pipelines::CSS)
      end
    end

    it "strips a relative_url_root before looking on disk" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/email.css"), Pipelines::CSS)
        use_rails_app(root: dir, relative_url_root: "/sub")

        expect(described_class.load("/sub/assets/email.css")).to eq(Pipelines::CSS)
      end
    end

    it "ignores the query string on a URL" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/email.css"), Pipelines::CSS)
        use_rails_app(root: dir)

        expect(described_class.load("/assets/email.css?body=1")).to eq(Pipelines::CSS)
      end
    end

    it "resolves a percent-escaped filename" do
      Dir.mktmpdir do |dir|
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        File.write(File.join(dir, "public/assets/my file.css"), Pipelines::CSS)
        use_rails_app(root: dir)

        expect(described_class.load("/assets/my%20file.css")).to eq(Pipelines::CSS)
      end
    end

    it "refuses to read outside public/" do
      Dir.mktmpdir do |dir|
        File.write(File.join(dir, "secret.css"), "h1 { color: leaked; }")
        FileUtils.mkdir_p(File.join(dir, "public/assets"))
        use_rails_app(root: dir)

        expect(described_class.load("/assets/../../secret.css")).to be_nil
      end
    end

    it "returns nil for a path that is not on disk" do
      Dir.mktmpdir do |dir|
        use_rails_app(root: dir)

        expect(described_class.load("/assets/missing.css")).to be_nil
      end
    end
  end

  # Railties is loaded in these specs but no application is booted, which is
  # exactly the state that used to raise from `::Rails.configuration`.
  describe "with railties loaded but no application" do
    it "Prefix falls back to a bare slash" do
      expect(CSSInline::Rails::CSSLoaders::Prefix.build).to eq("/")
    end

    it "PropshaftLoader declines" do
      expect(CSSInline::Rails::CSSLoaders::PropshaftLoader.load("/assets/a.css")).to be_nil
    end

    it "SprocketsLoader declines" do
      expect(CSSInline::Rails::CSSLoaders::SprocketsLoader.load("/assets/a.css")).to be_nil
    end

    it "FileSystemLoader declines" do
      Dir.mktmpdir { |dir| Dir.chdir(dir) { expect(described_class::FileSystemLoader.load("/a.css")).to be_nil } }
    end
  end
end
