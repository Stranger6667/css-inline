require 'json'
require "benchmark/ips"
require "premailer"
require_relative "../lib/css_inline"

file = File.read('../../benchmarks/benchmarks.json')
benchmarks = JSON.parse(file)

Premailer::Adapter.use = :nokogiri_fast

def premailer_inline(html)
  premailer = Premailer.new(html, with_html_string: true, :warn_level => Premailer::Warnings::SAFE)
  premailer.to_inline_css
end

benchmarks.each do |benchmark|
  Benchmark.ips do |x|
    x.report("css_inline_#{benchmark['name']}") { CSSInline::inline(benchmark['html']) }
    x.report("premailer_#{benchmark['name']}") { premailer_inline(benchmark['html']) }
    x.compare!
  end
end
