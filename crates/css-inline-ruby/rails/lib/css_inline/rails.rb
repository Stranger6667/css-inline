# frozen_string_literal: true

require "css_inline"
require "action_mailer"
require "active_support/ordered_options"
# `action_mailer` only autoloads it; `Hook` needs `Mail::Part`.
require "mail"

module CSSInline
  module Rails
    class Error < StandardError; end

    # Fresh each call, so mutating `config` cannot reach the defaults.
    def self.default_config
      ActiveSupport::OrderedOptions.new.merge!(
        strategies: %i[filesystem sprockets propshaft],
        inline_options: {}
      )
    end

    # No writer: the Railtie hands this object out as `config.css_inline`.
    class << self
      attr_reader :config
    end
    @config = default_config

    # Called by the Railtie. Apps that order interceptors themselves can call
    # it earlier; Mail ignores a second registration.
    def self.register_interceptors
      ActionMailer::Base.register_interceptor(Hook)
      ActionMailer::Base.register_preview_interceptor(Hook)
    end

    # `keep_at_rules` as premailer does: media queries cannot be inlined, and
    # dropping them breaks responsive and dark-mode mail. Not by default with
    # `inline_style_tags: false`: css_inline then drops every rule, `extra_css`
    # included.
    def self.inline(html)
      user_options = config.fetch(:inline_options, {})
      options = {keep_at_rules: user_options.fetch(:inline_style_tags, true)}
        .merge(user_options)
        .merge(load_remote_stylesheets: false)
      # Configured `extra_css` is kept, not replaced by what the links resolved to.
      extra_css = [options[:extra_css], CSSHelper.css_for_html(html)].compact.join("\n")
      options[:extra_css] = extra_css unless extra_css.empty?
      CSSInline.inline(html, **options)
    end
  end
end

require "css_inline/rails/version"
require "css_inline/rails/css_loaders"
require "css_inline/rails/css_helper"
require "css_inline/rails/hook"
require "css_inline/rails/railtie" if defined?(::Rails::Railtie)
