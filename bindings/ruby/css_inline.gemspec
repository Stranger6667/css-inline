# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name = "css_inline"
  spec.version = "0.16.0"
  spec.summary = "High-performance library for inlining CSS into HTML 'style' attributes"
  spec.description = <<-EOF
    `css_inline` inlines CSS into HTML documents, using components from Mozilla's Servo project.
    This process is essential for sending HTML emails as you need to use "style" attributes instead of "style" tags.
  EOF
  spec.files = [
    *Dir["lib/**/*.rb"],
    *Dir["ext/css_inline/src/**/*.rs"],
    "ext/css_inline/Cargo.toml",
    "ext/css_inline/Cargo.lock",
    "ext/css_inline/extconf.rb",
    "README.md",
    "Cargo.toml",
    "Cargo.lock",
  ].compact
  spec.extensions = ["ext/css_inline/extconf.rb"]
  spec.rdoc_options = ["--main", "README.rdoc", "--charset", "utf-8", "--exclude", "ext/"]
  spec.authors = ["Dmitry Dygalo"]
  spec.email = ["dmitry@dygalo.dev"]
  spec.homepage = "https://github.com/Stranger6667/css-inline"
  spec.license = "MIT"
  spec.metadata = {
    "bug_tracker_uri"   => "https://github.com/Stranger6667/css-inline/issues",
    "changelog_uri"     => "https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby/CHANGELOG.md",
    "source_code_uri"   => "https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby",
    "funding_uri" => "https://github.com/sponsors/Stranger6667/",
  }

  spec.requirements = ["Rust >= 1.82"]
  spec.required_ruby_version = ">= 3.2.0"
  spec.required_rubygems_version = ">= 3.3.26"

  spec.add_dependency "rb_sys", "~> 0.9.116"

  spec.add_development_dependency "rake-compiler", "~> 1.3.0"
  spec.add_development_dependency "benchmark-ips", "~> 2.10"
  spec.add_development_dependency "premailer", "~> 1.21"
  spec.add_development_dependency "roadie", "~> 5.2.1"
  spec.add_development_dependency "nokogiri", "~> 1.15"
end
