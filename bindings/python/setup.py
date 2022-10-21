import platform

from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="css_inline",
    version="0.8.4",
    description="Fast CSS inlining written in Rust",
    long_description=open("README.md", encoding="utf-8").read(),
    long_description_content_type="text/markdown",
    keywords="css inline html rust",
    author="Dmitry Dygalo",
    author_email="dadygalo@gmail.com",
    maintainer="Dmitry Dygalo",
    maintainer_email="dadygalo@gmail.com",
    python_requires=">=3.7",
    url="https://github.com/Stranger6667/css-inline/tree/master/bindings/python",
    license="MIT",
    rust_extensions=[
        RustExtension(
            "css_inline",
            py_limited_api=True,
            features=(
                [] if platform.python_implementation() == "PyPy" else ["pyo3/abi3"]
            ),
            rust_version=">=1.54.0",
        )
    ],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: MacOS :: MacOS X",
        "Operating System :: Microsoft :: Windows",
        "Operating System :: POSIX",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
        "Programming Language :: Rust",
    ],
    zip_safe=False,
)
