require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("css_inline/css_inline") do |r|
  def musl_target?
    return true if ENV['CARGO_BUILD_TARGET']&.include?('musl')

    if File.exist?('/etc/alpine-release') ||
       (`ldd --version 2>&1` =~ /musl/ rescue false)
      return true
    end

    host_target = `rustc -vV 2>/dev/null`.match(/host: (.+)/)[1] rescue nil
    return host_target&.include?('musl') || false
  end

  if musl_target?
    r.extra_rustflags = ['-C', 'target-feature=-crt-static']
  end
end
