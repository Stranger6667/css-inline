# frozen_string_literal: true

require_relative "lib/css_inline/rails/version"

Gem::Specification.new do |spec|
  spec.name = "css_inline-rails"
  spec.version = CSSInline::Rails::VERSION
  # Pure Ruby, unlike the `css_inline` gem it wraps.
  spec.platform = Gem::Platform::RUBY
  spec.summary = "Inline CSS into Rails emails with css_inline"
  spec.description = <<-EOF
    Hooks `css_inline` into ActionMailer so that stylesheets linked from your
    mailer views are inlined into "style" attributes on delivery.
  EOF
  # `git ls-files`, as `bundle gem` and both premailer-rails and roadie-rails
  # do, so a new file cannot be silently left out of the gem.
  spec.files = `git ls-files -z -- lib README.md CHANGELOG.md`.split("\x0")
  spec.require_paths = ["lib"]
  spec.authors = ["Dmitry Dygalo"]
  spec.email = ["dmitry@dygalo.dev"]
  spec.homepage = "https://github.com/Stranger6667/css-inline"
  spec.license = "MIT"
  spec.metadata = {
    "bug_tracker_uri" => "https://github.com/Stranger6667/css-inline/issues",
    "changelog_uri" => "https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby/rails/CHANGELOG.md",
    "source_code_uri" => "https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby/rails",
    "funding_uri" => "https://github.com/sponsors/Stranger6667/"
  }

  spec.required_ruby_version = ">= 3.2.0"

  spec.add_dependency "actionmailer", ">= 7.0"
  spec.add_dependency "css_inline", "~> 0.21"
  # Used read-only, to find `link` hrefs. Already present in any Rails app via
  # actionview -> rails-html-sanitizer -> loofah.
  spec.add_dependency "nokogiri", ">= 1.13"
end
