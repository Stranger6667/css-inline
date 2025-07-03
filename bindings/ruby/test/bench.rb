require 'json'
require "benchmark/ips"
require "premailer"
require "roadie"
require_relative "../lib/css_inline"

file = File.read('../../benchmarks/benchmarks.json')
benchmarks = JSON.parse(file)

Premailer::Adapter.use = :nokogiri_fast

def premailer_inline(html)
  premailer = Premailer.new(html, with_html_string: true, :warn_level => Premailer::Warnings::SAFE)
  premailer.to_inline_css
end

def roadie_inline(html)
  document = Roadie::Document.new(html)
  document.transform
end

def test_library(name, html, &block)
  begin
    result = block.call(html)
    [true, nil]
  rescue => e
    puts "Warning: #{name} failed with: #{e.class} - #{e.message}"
    [false, e]
  end
end

def format_time(seconds)
  case seconds
  when 0...0.000001
    "%.3f ns" % (seconds * 1_000_000_000)
  when 0.000001...0.001
    "%.3f μs" % (seconds * 1_000_000)
  when 0.001...1
    "%.3f ms" % (seconds * 1_000)
  else
    "%.3f s" % seconds
  end
end

benchmarks.each do |benchmark|
  puts "\n=== Benchmark: #{benchmark['name']} ==="

  # Test which libraries work for this benchmark
  libraries = {
    "css_inline" => -> (html) { CSSInline::inline(html) },
    "premailer" => -> (html) { premailer_inline(html) },
    "roadie" => -> (html) { roadie_inline(html) }
  }

  working_libraries = {}
  libraries.each do |name, func|
    success, error = test_library(name, benchmark['html'], &func)
    if success
      working_libraries[name] = func
    else
      puts "  Skipping #{name} for this benchmark due to error"
    end
  end

  if working_libraries.empty?
    puts "  No libraries could process this benchmark!"
    next
  end

  Benchmark.ips do |x|
    working_libraries.each do |name, func|
      x.report("#{name}_#{benchmark['name']}") do
          func.call(benchmark['html'])
      end
    end
    x.compare!
  end
end
