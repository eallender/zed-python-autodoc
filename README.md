# Python Autodoc for Zed

A Zed extension that automatically generates PEP 257-compliant docstrings for Python functions and classes.

## Features

- **Smart Docstring Generation**: Type `"""` after a function or class definition to trigger auto-completion
- **PEP 257 Compliant**: Follows Python's official docstring conventions
- **Type Hint Support**: Automatically extracts type annotations from function signatures
- **Snippet Integration**: Generated docstrings include tab stops for easy editing
- **Works with**:
  - Functions (sync and async)
  - Methods (instance, class, and static)
  - Classes
  - Multi-line function signatures
  - Default parameters
  - `*args` and `**kwargs`

## Development Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/eallender/zed-python-autodoc.git
   cd zed-python-autodoc
   ```

2. Build the extension and LSP server:
   ```bash
   # Build the WASM extension
   cargo build --release --target wasm32-wasip1

   # Build the LSP server
   cd crates/lsp-server
   cargo build --release
   cd ../..
   ```

3. Install as dev extension in Zed:
   - Open Zed
   - Press `Cmd+Shift+P` / `Ctrl+Shift+P`
   - Select "zed: install dev extension"
   - Navigate to this repository directory

## Usage

1. Open a Python file in Zed
2. Position your cursor right after a function or class definition (after the `:`)
3. Type `"""` (three double quotes)
4. Accept the completion to generate a docstring with:
   - `${1:Summary.}` placeholder (press Tab to edit)
   - **Args:** section for function parameters (if any)
   - **Returns:** section for return types (if not `None`)

### Example

**Before:**
```python
def greet(name: str, greeting: str = "Hello") -> str:
    """
```

**After accepting completion:**
```python
def greet(name: str, greeting: str = "Hello") -> str:
    """
    Summary.

    Args:
        name (str): Description.
        greeting (str), optional (default: "Hello"): Description.

    Returns:
        str: Description.
    """
```

### PEP 257 Compliance

The extension follows [PEP 257](https://peps.python.org/pep-0257/) conventions:

- **Class docstrings**: Summary only (describes the class purpose)
- **Method/Function docstrings**: Summary + Args + Returns
- **`__init__` docstrings**: Constructor parameters are documented in the `__init__` method's docstring

## License

MIT License - see [LICENSE](LICENSE) for details.
