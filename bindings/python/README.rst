css_inline
==========

|Build| |Version| |Python versions| |License|

Blazing-fast CSS inlining for Python implemented with Mozilla's Servo project components.

Features:

- Removing ``style`` tags after inlining;
- Resolving external stylesheets (including local files);
- Control if ``style`` tags should be processed;
- Additional CSS to inline;
- Inlining multiple documents in parallel (via Rust-level threads)

The project supports CSS Syntax Level 3.

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
    # HTML becomes this:
    #
    # <html>
    # <head>
    #    <title>Test</title>
    #    <style>h1 { color:blue; }</style>
    # </head>
    # <body>
    #     <h1 style="color:blue;">Big Text</h1>
    # </body>
    # </html>

If you want to inline many HTML documents, you can utilize ``inline_many`` that processes the input in parallel.

.. code:: python

    import css_inline

    css_inline.inline_many(["<...>", "<...>"])

``inline_many`` will use Rust-level threads; thus, you can expect it's running faster than ``css_inline.inline`` via Python's ``multiprocessing`` or ``threading`` modules.

For customization options use the ``CSSInliner`` class:

.. code:: python

    import css_inline

    inliner = css_inline.CSSInliner(remove_style_tags=True)
    inliner.inline("...")

Performance
-----------

Due to the usage of efficient tooling from Mozilla's Servo project (``html5ever``, ``rust-cssparser`` and others) this
library has excellent performance characteristics. In comparison with other Python projects, it is ~6-15x faster than the nearest alternative.

For inlining CSS in the html document from the ``Usage`` section above we have the following breakdown in our benchmarks:

- ``css_inline 0.7.0`` - 25.21 us
- ``premailer 3.7.0`` - 340.89 us (**x13.52**)
- ``inlinestyler 0.2.4`` - 2.44 ms (**x96.78**)
- ``pynliner 0.8.0`` - 2.78 ms (**x110.27**)

And for a more realistic email:

- ``css_inline 0.6.0`` - 529.1 us
- ``premailer 3.7.0`` - 3.38 ms (**x6.38**)
- ``inlinestyler 0.2.4`` - 64.41 ms (**x121.73**)
- ``pynliner 0.8.0`` - 93.11 ms (**x175.97**)

You can take a look at the benchmarks' code at ``benches/bench.py`` file.
The results above were measured with stable ``rustc 1.47.0``, ``Python 3.8.6`` on i8700K, and 32GB RAM.

Python support
--------------

``css_inline`` supports Python 3.6, 3.7, 3.8, 3.9, and 3.10.

Extra materials
---------------

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- `Rust crate <https://dygalo.dev/blog/rust-for-a-pythonista-2/>`_
- `Python bindings <https://dygalo.dev/blog/rust-for-a-pythonista-3/>`_

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
