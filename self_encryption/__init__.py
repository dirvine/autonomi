try:
    from . import _self_encryption
    from _self_encryption import *
    from .cli import cli
except ImportError as e:
    import sys
    print(f"Error importing self_encryption: {e}", file=sys.stderr)
    raise

__all__ = ['cli']