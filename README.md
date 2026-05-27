# Python Autodoc for Zed

Generates PEP 257 docstrings for Python functions and classes. Type `"""` on the line after a definition to trigger completion.

Handles typed parameters, return types, exceptions, dataclasses, async functions, `*args`/`**kwargs`, and nested functions.

## Examples

**Function with typed parameters:**
```python
def greet(name: str, greeting: str = "Hello") -> str:
    """
    Summary.

    Args:
        name (str): Description.
        greeting (str): Description, optional (default: "Hello").

    Returns:
        str: Description.
    """
```

**Function with exceptions:**
```python
def divide(a: float, b: float) -> float:
    """
    Summary.

    Args:
        a (float): Description.
        b (float): Description.

    Returns:
        float: Description.

    Raises:
        ZeroDivisionError: Description.
    """
    if b == 0:
        raise ZeroDivisionError("Cannot divide by zero")
    return a / b
```

**Dataclass:**
```python
@dataclass
class Point:
    """
    Summary.

    Attributes:
        x (float): Description.
        y (float): Description.
    """
    x: float
    y: float
```

More examples can be found in [examples/](examples/).

PEP 257 notes:
- Class docstrings get a summary only; `__init__` parameters are documented in `__init__`
- `None` return types are omitted
- `Raises:` is only generated when the function body contains `raise` statements

## License

MIT — see [LICENSE](LICENSE).
