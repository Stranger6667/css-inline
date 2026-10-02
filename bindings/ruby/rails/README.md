# css_inline-rails

Inlines CSS into your Rails emails with [`css_inline`](https://github.com/Stranger6667/css-inline),
50-100x faster than `premailer` and `roadie` in [our benchmarks](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby#performance).

```ruby
# Gemfile
gem "css_inline-rails"
```

Adding the gem registers an ActionMailer interceptor and a preview interceptor. Every
outgoing HTML message, `deliver_now` or `deliver_later`, and every mailer preview gets
its `<style>` blocks and linked stylesheets inlined into `style` attributes. Text-only
messages pass through untouched.

Requires Ruby 3.2+ and Rails 7.1+.

## Migrating from premailer-rails

### No generated text part

`premailer-rails` generates a `text/plain` part for every HTML-only message unless you
set `generate_text_part: false`. This gem has no such option yet.

If your mailers render only `format.html`, they will send HTML-only mail after the
swap. SpamAssassin's `MIME_HTML_ONLY` rule scores such mail, and any spec that reads
`text_part` or `parts` starts failing on `nil`. Add a text template next to each HTML
one (`welcome.text.erb`) and render both formats:

```ruby
mail(to: user.email) do |format|
  format.text
  format.html
end
```

### Renames

Replace these across `app/` and `config/` in one pass. The old names do nothing here:
a leftover `skip_premailer` header goes out to the recipient, and the message gets
inlined anyway.

| premailer-rails | css_inline-rails |
|---|---|
| `skip_premailer: true` | `skip_css_inline: true` |
| `<style data-premailer="ignore">` | `<style data-css-inline="ignore">` plus `keep_style_tags: true` (see below) |
| `<link data-premailer="ignore">` | `<link data-css-inline="ignore">`, which removes the tag. `keep_link_tags: true` keeps every `<link>`, inlined ones included |
| `Premailer::Rails::Hook.perform(mail)` | `CSSInline::Rails::Hook.perform(mail)` |
| `Premailer::Rails.register_interceptors` | `CSSInline::Rails.register_interceptors` |

premailer leaves an ignored `<style>` in the output without inlining it.
`data-css-inline="ignore"` skips inlining but deletes the block, and
`keep_style_tags: true` brings it back, along with every other `<style>` block.
`data-css-inline="keep"` is not a substitute: it keeps the block but still inlines its
rules, so a broad rule such as `a { color: inherit }` overrides your linked stylesheet.

Delete `config/initializers/premailer*.rb`: the app fails to boot with
`uninitialized constant Premailer` while it references the old gem. A custom premailer
strategy keeps working unchanged, since both gems call `load(url)` on it:

```ruby
config.css_inline.strategies = [MyPremailerStrategy]
```

### Not supported

- `generate_text_part`: see above.
- The `:network` strategy: remote links are skipped. A Vite dev server URL such as
  `/vite-dev/styles/email.css` raises `FileNotFound`. Add a
  [custom strategy](#custom-strategies) that reads it from the dev server, or skip the
  link in development with `data-css-inline="ignore"`.
- `-premailer-width`, `-premailer-cellpadding` and other `-premailer-*` properties: they
  stay in the `style` attribute instead of becoming HTML attributes. Write
  `width="100%"` and friends in the template.
- `remove_comments` and `remove_ids`: comments and ids stay in the output.
- Rules `css_inline` cannot inline, such as `:hover` or `::after`, are dropped.
  premailer keeps them in a `<style>` block. Only `@`-rules survive, through
  `keep_at_rules`. Put the rules you need in a `<style data-css-inline="keep">` block.

## How stylesheets are found

Mailer layouts link stylesheets like this:

```erb
<%= stylesheet_link_tag "email" %>
```

That renders a digested path such as `/assets/email-8f3a1c.css`, or
`https://cdn.example.com/assets/email-8f3a1c.css` with an asset host. Only the asset
pipeline can resolve it, so the gem resolves it from the URL path and passes the CSS to
`css_inline` as `extra_css`. It tries three strategies in order:

1. `:filesystem` reads `public/`, where `assets:precompile` puts assets in production.
2. `:sprockets` asks the Sprockets manifest.
3. `:propshaft` asks Propshaft, so development works without precompiling.

If no strategy resolves a link without a host, `deliver_now` raises
`CSSInline::Rails::CSSHelper::FileNotFound` to the caller and sends nothing. With
`deliver_later` the job fails and ActiveJob's retry rules apply. A mailer preview shows
the error. All errors inherit from `CSSInline::Rails::Error`.

The gem skips a link with a host that no strategy resolves, such as Google Fonts, and
never fetches it over the network, because a CDN outage would then fail your mail.
`css_inline` removes those `<link>` tags too. To keep the font, set
`inline_options: { keep_link_tags: true }` or link a local copy.

### Caching

When your application does not reload code (`config.enable_reloading = false`, the
production default), the gem caches each stylesheet by URL for the life of the process.
A digested URL changes whenever the file does, and a restart clears the cache. With
reloading on, as in development, it reads stylesheets on every message.

### Custom strategies

A strategy is any object that responds to `load(url)` and returns the CSS as a string,
or `nil` to let the next strategy try:

```ruby
module S3Loader
  def self.load(url)
    S3_BUCKET.object(URI.parse(url).path.delete_prefix("/")).get.body.read
  rescue Aws::S3::Errors::NoSuchKey
    nil
  end
end

config.css_inline.strategies = [:filesystem, S3Loader]
```

## Media queries and kept tags

Media queries cannot be inlined. The gem sets `keep_at_rules: true`, so `css_inline`
inlines the rest of each stylesheet and keeps `@media` rules, including those from
linked files, in a `<style>` block, as premailer does. Set `keep_at_rules: false` in
`inline_options` to drop them.

`data-css-inline="ignore"` works on any element and skips inlining into it. On a
`<style>` or `<link>` it also removes the tag. `data-css-inline="keep"` inlines a
`<style>` block's rules and also leaves the block in the output, for clients that
support `<style>`:

```erb
<style data-css-inline="keep">
  a:hover { text-decoration: underline; }
</style>
```

## Configuration

Set options in `config/application.rb` or an environment file:

```ruby
config.css_inline.inline_options = { keep_style_tags: true }
config.css_inline.strategies = %i[filesystem propshaft]
```

| Key | Default | Meaning |
|---|---|---|
| `inline_options` | `{}` | Passed to `CSSInline.inline`, see [the options](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby#configuration) |
| `strategies` | `%i[filesystem sprockets propshaft]` | Tried in order for each linked stylesheet |

The gem passes `keep_at_rules: true` unless `inline_options` overrides it, and always
passes `load_remote_stylesheets: false`, since it resolves links itself. An `extra_css`
you configure comes before the linked CSS.

`CSSInline::Rails.config` is the same object, for code outside the Rails config.

Skip a single message:

```ruby
mail(to: "user@example.com", skip_css_inline: true)
```

## License

MIT
