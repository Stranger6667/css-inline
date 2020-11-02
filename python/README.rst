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

If you want to inline many HTML documents, you can utilize ``inline_many`` that processes the input in parallel.

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

- ``css_inline 0.6.0`` - 23.01 us
- ``premailer 3.7.0`` - 340.89 us (**x14.81**)
- ``inlinestyler 0.2.4`` - 2.44 ms (**x106.35**)
- ``pynliner 0.8.0`` - 2.78 ms (**x121.04**)

And for a more realistic email:

- ``css_inline 0.6.0`` - 483.3 us
- ``premailer 3.7.0`` - 3.38 ms (**x7**)
- ``inlinestyler 0.2.4`` - 64.41 ms (**x133.28**)
- ``pynliner 0.8.0`` - 93.11 ms (**x192.65**)

You can take a look at the benchmarks' code at ``benches/bench.py`` file.
The results above were measured with stable ``rustc 1.47.0``, ``Python 3.8.6`` on i8700K, and 32GB RAM.

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
