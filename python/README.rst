css_inline
==========

|Build| |Version| |Python versions| |License|

Fast CSS inlining for Python implemented in Rust.

Features:

- Removing ``style`` tags after inlining;
- Resolving external stylesheets (including local files);
- Control if ``style`` tags should be processed;
- Additional CSS to inline;

Installation
------------

To install ``css_inline`` via ``pip`` run the following command:

.. code:: bash

    pip install css_inline

Usage
-----

To inline CSS in a HTML document:

.. code:: python

    import css_inline

    HTML = """<html>
    <head>
        <title>Test</title>
        <style>h1 { color:blue; }</style>
    </head>
    <body>
        <h1>Big Text</h1>
    </body>
    </html>"""

    inlined = css_inline.inline(HTML)

If you want to inline many HTML documents then you can utilize ``inline_many`` that processes the input in parallel.

.. code:: python

    import css_inline

    css_inline.inline_many(["...", "..."])

For customization options use ``CSSInliner`` class:

.. code:: python

    import css_inline

    inliner = css_inline.CSSInliner(remove_style_tags=True)
    inliner.inline("...")

Performance
-----------

Due to the usage of efficient tooling from Mozilla's Servo project (``html5ever``, ``rust-cssparser`` and others) this
library has good performance characteristics. In comparison with other Python projects, it is ~7-15x faster than the nearest competitor.

For inlining CSS in the html document from the ``Usage`` section above we have the following breakdown in our benchmarks:

- ``css_inline 0.5.0`` - 22.88 us
- ``premailer 3.7.0`` - 331 us (**x14.48**)
- ``inlinestyler 0.2.4`` - 2.43 ms (**x106.43**)
- ``pynliner 0.8.0`` - 2.76 ms (**x120.76**)

And for a more realistic email:

- ``css_inline 0.5.0`` - 469.57 us
- ``premailer 3.7.0`` - 3.32 ms (**x7.09**)
- ``inlinestyler 0.2.4`` - 63.15 ms (**x125.58**)
- ``pynliner 0.8.0`` - 90.66 ms (**x189.20**)

You can take a look at the benchmarks' code at ``benches/bench.py`` file.
The results above were measured with stable ``rustc 1.45.2``, ``Python 3.8`` on i8700K, and 32GB RAM.

Python support
--------------

``css_inline`` supports Python 3.5, 3.6, 3.7, and 3.8.

License
-------

The code in this project is licensed under `MIT license`_.
By contributing to ``css_inline``, you agree that your contributions
will be licensed under its MIT license.

.. |Build| image:: https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg
   :target: https://github.com/Stranger6667/css_inline/actions
.. |Version| image:: https://img.shields.io/pypi/v/css_inline.svg
   :target: https://pypi.org/project/css_inline/
.. |Python versions| image:: https://img.shields.io/pypi/pyversions/css_inline.svg
   :target: https://pypi.org/project/css_inline/
.. |License| image:: https://img.shields.io/pypi/l/css_inline.svg
   :target: https://opensource.org/licenses/MIT

.. _MIT license: https://opensource.org/licenses/MIT
