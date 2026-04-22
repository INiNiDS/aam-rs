"""aam_py — Python bindings for the aam-rs AAML configuration parser.

The compiled Rust extension is ``aam_rs``. This package re-exports the
current AAM API for compatibility with ``import aam_py``.

Basic usage::

    from aam_py import AAM, AAMBuilder, SchemaField

    # Parse from string
    aam = AAM.parse("name = Alice\\host = localhost")
    print(aam["name"])  # Alice

    # Load from file
    aam = AAM.load("config.aam")
    for key, value in aam.items():
        print(f"{key}: {value}")
"""

from .aam_py import AAM, AAMBuilder, SchemaField, __version__  # noqa: F401

__all__ = ["AAM", "AAMBuilder", "SchemaField", "__version__"]
