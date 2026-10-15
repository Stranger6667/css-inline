# frozen_string_literal: true

begin
  require "css_inline/#{RUBY_VERSION.to_f}/css_inline"
rescue LoadError
  require "css_inline/css_inline"
end
