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

from typing import Dict, Optional, Tuple

from .aam_py import AAM, AAMBuilder, SchemaField, __version__ as _native_version  # noqa: F401


def _parse_section_header(line: str) -> Optional[str]:
    if not line.startswith("#"):
        return None
    rest = line[1:].strip()
    return rest if rest.endswith(".aam") else None


def _parse_assignment(line: str) -> Optional[Tuple[str, str]]:
    key, sep, value = line.partition("=")
    if not sep:
        return None
    key = key.strip()
    if not key:
        return None
    return key, value.strip()


def split_aam(content: str) -> Dict[str, AAMBuilder]:
    result: Dict[str, AAMBuilder] = {}
    current_name = None
    current_builder = None

    for raw_line in content.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        header = _parse_section_header(line)
        if header is not None:
            if current_name is not None and current_builder is not None:
                result[current_name] = current_builder
            current_name = header
            current_builder = AAMBuilder()
            continue

        if current_name is not None and current_builder is not None:
            assignment = _parse_assignment(line)
            if assignment is not None:
                current_builder.add_line(*assignment)

    if current_name is not None and current_builder is not None:
        result[current_name] = current_builder

    return result

__all__ = ["AAM", "AAMBuilder", "SchemaField", "__version__", "split_aam"]

__version__ = "2.4.1"

