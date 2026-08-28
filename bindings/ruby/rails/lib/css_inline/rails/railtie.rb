# frozen_string_literal: true

module CSSInline
  module Rails
    class Railtie < ::Rails::Railtie
      ActiveSupport.on_load(:action_mailer) do
        CSSInline::Rails.register_interceptors
      end
    end
  end
end
