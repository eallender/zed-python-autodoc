# Python Autodoc for Zed
<p align="center">
  <img src="https://img.shields.io/badge/Zed-Extension-7C3AED?logo=zedindustries" alt="Zed Extension" />
  <img src="https://img.shields.io/github/downloads/eallender/zed-python-autodoc/total" alt="Downloads" />
  <img src="https://img.shields.io/github/v/release/eallender/zed-python-autodoc" alt="Release" />
</p>

Generates PEP 257 docstrings for Python functions and classes. Type `"""` on the line after a definition to trigger completion.

Handles typed parameters, return types, exceptions, dataclasses, async functions, `*args`/`**kwargs`, and nested functions.

## Examples

1. **Function with typed parameters**

![Basic Example](assets/standard.gif)   

2. **Function with exceptions**

![Exception Example](assets/raises.gif)   

3. **Dataclasses**

![Basic Example](assets/dataclass.gif)

4. **Complex signatures**

![Complex Example](assets/complex.gif)   

More examples can be found in [examples/](examples/).

**PEP 257 notes**:
- Class docstrings get a summary only; `__init__` parameters are documented in `__init__`
- `None` return types are omitted
- `Raises:` is only generated when the function body contains `raise` statements

## License

MIT — see [LICENSE](LICENSE).
