# frozen_string_literal: true

require "css_inline"
require "action_mailer"
# `action_mailer` only autoloads it; `Hook` needs `Mail::Part`.
require "mail"

require "css_inline/rails/version"
require "css_inline/rails/css_loaders"
require "css_inline/rails/css_helper"
require "css_inline/rails/hook"

module CSSInline
  module Rails
    # Fresh each call, so mutating `config` cannot reach the defaults.
    def self.default_config
      {
        strategies: %i[filesystem sprockets propshaft],
        inline_options: {}
      }
    end

    class << self
      attr_accessor :config
    end
    self.config = default_config

    def self.register_interceptors
      ActionMailer::Base.register_interceptor(Hook)
      return unless ActionMailer::Base.respond_to?(:register_preview_interceptor)

      ActionMailer::Base.register_preview_interceptor(Hook)
    end

    def self.inline(html)
      options = config.fetch(:inline_options, {}).merge(load_remote_stylesheets: false)
      # Configured `extra_css` is kept, not replaced by what the links resolved to.
      extra_css = [options[:extra_css], CSSHelper.css_for_html(html)].compact.join("\n")
      options[:extra_css] = extra_css unless extra_css.empty?
      CSSInline.inline(html, **options)
    end
  end
end

require "css_inline/rails/railtie" if defined?(::Rails::Railtie)
