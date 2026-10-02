# frozen_string_literal: true

module CSSInline
  module Rails
    class Railtie < ::Rails::Railtie
      # The same object, so `config.css_inline.inline_options = {...}` in
      # `config/application.rb` or an environment file configures the gem.
      config.css_inline = CSSInline::Rails.config

      initializer "css_inline.register_interceptors" do
        ActiveSupport.on_load(:action_mailer) { CSSInline::Rails.register_interceptors }
      end
    end
  end
end
