"""aam_rs — compatibility shim for the aam-py Python bindings.

The compiled Rust extension is ``aam_py``.  This package re-exports
everything from there so users can use whichever name they prefer::

    from aam_py import AAML   # primary
    from aam_rs import AAML   # backward-compatible alias
"""

from aam_py import AAML, __version__  # noqa: F401

__all__ = ["AAML", "__version__"]
