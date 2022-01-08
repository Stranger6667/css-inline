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
library has excellent performance characteristics. In comparison with other Python projects, it is ~9-21x faster than the nearest alternative.

For inlining CSS in the html document from the ``Usage`` section above we have the following breakdown in our benchmarks:

- ``css_inline 0.7.8`` - 21.76 us
- ``premailer 3.10.0`` - 461.54 us (**x21.21**)
- ``toronado 0.1.0`` - 1.87 ms (**x85.93**)
- ``inlinestyler 0.2.4`` - 2.85 ms (**x130.97**)
- ``pynliner 0.8.0`` - 3.34 ms (**x153.49**)

And for a more realistic email:

- ``css_inline 0.7.8`` - 433.39 us
- ``premailer 3.10.0`` - 3.9 ms (**x9.01**)
- ``toronado 0.1.0`` - 43.89 ms (**x101.27**)
- ``inlinestyler 0.2.4`` - 75.77 ms (**x174.83**)
- ``pynliner 0.8.0`` - 123.6 ms (**x285.19**)

You can take a look at the benchmarks' code at ``benches/bench.py`` file.
The results above were measured with stable ``rustc 1.57.0``, ``Python 3.9.9`` on i8700K, and 32GB RAM.

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
