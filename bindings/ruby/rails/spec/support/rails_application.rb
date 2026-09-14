# frozen_string_literal: true

require "rails"
require "active_support/all"
# Each reopens Rails::Application with an accessor the loaders read: `assets`
# from Propshaft, `assets_manifest` from sprockets-rails.
require "propshaft"
require "sprockets/railtie"

# A real, un-booted Rails::Application. Every accessor used is public Rails API,
# so nothing here is stubbed.
module RailsApplication
  APP = Class.new(::Rails::Application).instance
  ORIGINAL_ROOT = APP.config.root

  # Clear what defining the class set, so examples start with no application.
  ::Rails.app_class = nil
  ::Rails.application = nil

  def use_rails_app(assets: nil, assets_manifest: nil, prefix: "/assets",
                    relative_url_root: nil, root: nil)
    APP.assets = assets
    APP.assets_manifest = assets_manifest
    APP.config.root = root if root
    APP.config.assets.prefix = prefix
    APP.config.relative_url_root = relative_url_root
    ::Rails.application = APP
    APP
  end
end

RSpec.configure do |config|
  config.include RailsApplication

  # `config.assets` is shared railtie state, so changes must be undone.
  config.around do |example|
    original_env = ::Rails.env.to_s
    example.run
  ensure
    ::Rails.application = nil
    ::Rails.app_class = nil
    ::Rails.env = original_env
    RailsApplication::APP.assets = nil
    RailsApplication::APP.assets_manifest = nil
    RailsApplication::APP.config.root = RailsApplication::ORIGINAL_ROOT
    RailsApplication::APP.config.assets.prefix = "/assets"
    RailsApplication::APP.config.relative_url_root = nil
  end
end
