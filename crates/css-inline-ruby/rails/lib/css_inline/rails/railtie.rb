# frozen_string_literal: true

module CSSInline
  module Rails
    class Railtie < ::Rails::Railtie
      # The same object, so `config.css_inline.inline_options = {...}` in
      # `config/application.rb` or an environment file configures the gem.
      config.css_inline = CSSInline::Rails.config

      # `config.css_inline = config_for(:css_inline)` swaps in another object.
      config.after_initialize do |app|
        configured = app.config.css_inline
        app.config.css_inline = CSSInline::Rails.config.merge!(configured) unless configured.equal?(CSSInline::Rails.config)
      end

      # Not an `initializer`: those never run for a gem required after boot,
      # e.g. `require: false` plus a require in `config/initializers`.
      ActiveSupport.on_load(:action_mailer) { CSSInline::Rails.register_interceptors }
    end
  end
end
