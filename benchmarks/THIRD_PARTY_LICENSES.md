# Third-party content in `benchmarks.json`

These entries in `benchmarks.json` are third-party templates, used only as benchmark inputs.
All are under the MIT License; the copyright notices and licence text follow.
Each entry is the upstream file with remote `<link rel="stylesheet">` tags removed so benchmarks never touch the network, plus the listed changes.

- `postmark_receipt`: [ActiveCampaign/postmark-templates `templates/basic/receipt/content.html`](https://github.com/ActiveCampaign/postmark-templates/blob/fa73527acb4eb59cf673a54572d213e908f1515c/templates/basic/receipt/content.html). Copyright (c) 2015 Wildbit.
- `cerberus_hybrid`: [emailmonday/Cerberus `cerberus-hybrid.html`](https://github.com/emailmonday/Cerberus/blob/af3cec9dd12fbbea0c6e9a1b0bf82a0479726957/cerberus-hybrid.html). Copyright (c) 2017 Ted Goas.
- `foundation_hero`: [foundation/foundation-emails `templates/hero.html`](https://github.com/foundation/foundation-emails/blob/7a37ae4cad4f97068f8257ba92a67a261080e3c5/templates/hero.html) and `dist/foundation-emails.css` from [foundation-emails 2.4.0](https://github.com/foundation/foundation-emails/tree/371f3becfdd03e943c772039fff09b1a57928762). Copyright (c) 2017 Foundation, inc. Changes: Inky tags expanded to HTML tables by a minimal Python port of Inky (not the official Inky compiler); `foundation-emails.css` embedded in `<style>`.
- `mjml_colorlib_1`: [ColorlibHQ/email-templates `1/index.html`](https://github.com/ColorlibHQ/email-templates/blob/3018557fe943fb1c3e3367aa52d39b2fca44f725/1/index.html). Colorlib (the upstream `LICENSE-MIT` has no copyright line).
- `bootstrap_cheatsheet`: `cheatsheet/index.html` from [Bootstrap 5.3.3 examples](https://github.com/twbs/bootstrap/releases/download/v5.3.3/bootstrap-5.3.3-examples.zip) ([v5.3.3](https://github.com/twbs/bootstrap/tree/6e1f75f420f68e1d52733b8e407fc7c3766c9dba)). Copyright (c) 2011-2024 The Bootstrap Authors. Changes: `bootstrap.min.css` and the page CSS embedded in `<style>`.

## MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
