# css_inline-rails

Inlines CSS into your Rails emails using [`css_inline`](https://github.com/Stranger6667/css-inline),
which is 50-100x faster than `premailer` and `roadie`.

```ruby
# Gemfile
gem "css_inline-rails"
```

That is the whole setup. An ActionMailer interceptor is registered automatically, and
every outgoing HTML message gets its `<style>` blocks and linked stylesheets inlined
into `style` attributes.

## Migrating from premailer-rails

**`premailer-rails` generates a `text/plain` part for you by default. This gem does not.**

If your mailers rely on `generate_text_part` and you swap gems without adding explicit
`text_part` blocks, your emails go out HTML-only. That is a deliverability regression
that no test will catch, so check your mailers before switching.

Everything else maps over:

| premailer-rails | css_inline-rails |
|---|---|
| `skip_premailer` header | `skip_css_inline` header |
| `data-premailer="ignore"` | `data-css-inline="ignore"` — but see below |
| `generate_text_part` | not supported — write a `text_part` |
| `:network` strategy | not supported — remote links are skipped |

## How stylesheets are found

Mailer views usually link a stylesheet rather than inlining a `<style>` block:

```erb
<%= stylesheet_link_tag "email" %>
```

That renders a digested path such as `/assets/email-8f3a1c.css`, which only the asset
pipeline can resolve. This gem resolves it locally, then hands the result to `css_inline`
as `extra_css`. Three strategies are tried in order:

1. `:filesystem` — reads `public/`, where precompiled assets already live in production.
2. `:sprockets` — the Sprockets manifest.
3. `:propshaft` — Propshaft, the default pipeline since Rails 8.

If none of them resolve a linked stylesheet, `CSSHelper::FileNotFound` is raised rather
than silently sending unstyled mail.

Remote stylesheets (`https://cdn.example.com/email.css`) are **not** fetched — downloading
over the network while rendering mail is a footgun. They are skipped, not treated as an
error, so a CDN font sheet will not fail a delivery. Link a local asset if you need it inlined.

`data-css-inline="ignore"` on a `link` differs from `data-premailer="ignore"`: premailer
leaves the tag in the output, `css_inline` still removes it. The stylesheet is therefore
neither inlined nor delivered. Set `keep_link_tags: true` in `inline_options` if you need
the tag to survive.

## Configuration

```ruby
# config/initializers/css_inline.rb
CSSInline::Rails.config[:inline_options] = { keep_style_tags: true }
CSSInline::Rails.config[:strategies] = %i[filesystem propshaft]
```

`inline_options` is passed to `CSSInline.inline`; see the
[`css_inline` options](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby#options).
`load_remote_stylesheets` is always forced off, because the strategies above have already
resolved every link.

Skip a single message:

```ruby
mail(to: "user@example.com", skip_css_inline: true)
```

## License

MIT
