# frozen_string_literal: true

require "tmpdir"
require "fileutils"
require "active_support/all"
require "propshaft"
require "sprockets"
require "sprockets/manifest"

# Real Propshaft and Sprockets objects over a throwaway asset directory, so a
# change in either breaks these specs.
module Pipelines
  CSS = "h1 { color: blue; }\n"

  def with_propshaft(css: CSS, prefix: "/assets")
    Dir.mktmpdir do |dir|
      source = File.join(dir, "app/assets/stylesheets")
      FileUtils.mkdir_p(source)
      File.write(File.join(source, "email.css"), css)

      config = ActiveSupport::OrderedOptions.new
      config.paths = [source]
      config.version = "1"
      config.prefix = prefix
      config.output_path = Pathname.new(File.join(dir, "public", prefix))
      config.manifest_path = Pathname.new(File.join(dir, "public", prefix, ".manifest.json"))
      config.compilers = [["text/css", Propshaft::Compiler::CssAssetUrls]]
      config.file_watcher = nil
      config.integrity_hash_algorithm = nil

      assembly = Propshaft::Assembly.new(config)
      # Asked of Propshaft, not hardcoded, so the spec tracks its digest format.
      digested = assembly.load_path.find("email.css").digested_path
      yield assembly, File.join(prefix, digested.to_s)
    end
  end

  def with_sprockets(css: CSS, prefix: "/assets")
    Dir.mktmpdir do |dir|
      source = File.join(dir, "app/assets/stylesheets")
      FileUtils.mkdir_p(source)
      File.write(File.join(source, "email.css"), css)

      environment = Sprockets::Environment.new(dir)
      environment.append_path(source)
      manifest = Sprockets::Manifest.new(environment, File.join(dir, "public", prefix))
      digested = environment["email.css"].digest_path

      yield manifest, File.join(prefix, digested.to_s)
    end
  end
end

# The documented strategy contract: `#load(url)` returning CSS or nil.
class RecordingStrategy
  attr_reader :urls

  def initialize(css = nil)
    @css = css
    @urls = []
  end

  def load(url)
    @urls << url
    @css.respond_to?(:call) ? @css.call(url) : @css
  end
end

RSpec.configure { |config| config.include Pipelines }
