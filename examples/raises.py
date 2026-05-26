"""Example file demonstrating Raises section support."""

from typing import Optional


# Test 1: Basic function with raise
def divide(a: int, b: int) -> float:
    """
    Summary.

    Args:
        a (int): Description.
        b (int): Description.

    Returns:
        float: Description.

    Raises:
        ZeroDivisionError: Description.
    """
    if b == 0:
        raise ZeroDivisionError("Cannot divide by zero")
    return a / b


# Test 2: Function without raises (should NOT have Raises section)
def add(a: int, b: int) -> int:
    """
    Summary.

    Args:
        a (int): Description.
        b (int): Description.

    Returns:
        int: Description.
    """
    return a + b


# Test 3: Multiple different exceptions
def process_data(data: str, max_length: int = 100) -> dict:
    """
    Summary.

    Args:
        data (str): Description.
        max_length (int): Description, optional (default: 100).

    Returns:
        dict: Description.

    Raises:
        ValueError: Description.
        TypeError: Description.
        RuntimeError: Description.
    """
    if not data:
        raise ValueError("Data cannot be empty")

    if not isinstance(data, str):
        raise TypeError("Data must be a string")

    if len(data) > max_length:
        raise RuntimeError("Data exceeds maximum length")

    return {"processed": data}


# Test 4: Raises with None return type
def validate_positive(value: int) -> None:
    """
    Summary.

    Args:
        value (int): Description.

    Raises:
        ValueError: Description.
    """
    if value < 0:
        raise ValueError("Value must be positive")


# Test 5: Nested raise in if/else
def check_range(value: int, min_val: int, max_val: int) -> bool:
    """
    Summary.

    Args:
        value (int): Description.
        min_val (int): Description.
        max_val (int): Description.

    Returns:
        bool: Description.

    Raises:
        ValueError: Description.
    """
    if value < min_val:
        raise ValueError(f"Value {value} below minimum {min_val}")
    elif value > max_val:
        raise ValueError(f"Value {value} above maximum {max_val}")
    return True


# Test 6: Raise from another exception
def parse_json(data: str) -> dict:
    """
    Summary.

    Args:
        data (str): Description.

    Returns:
        dict: Description.

    Raises:
        ValueError: Description.
    """
    import json

    try:
        return json.loads(data)
    except json.JSONDecodeError as e:
        raise ValueError("Invalid JSON") from e


# Test 7: Complex function with Args, Returns, and Raises
def fetch_user(user_id: int, include_inactive: bool = False) -> dict:
    """
    Summary.

    Args:
        user_id (int): Description.
        include_inactive (bool): Description, optional (default: False).

    Returns:
        dict: Description.

    Raises:
        ValueError: Description.
        TypeError: Description.
    """
    if user_id <= 0:
        raise ValueError("User ID must be positive")

    if not isinstance(user_id, int):
        raise TypeError("User ID must be an integer")

    # Simulate database lookup
    user = {"id": user_id, "name": "Test"}
    return user


# Test 8: Async function with raises
async def fetch_url(url: str, timeout: int = 30) -> str:
    """
    Summary.

    Args:
        url (str): Description.
        timeout (int): Description, optional (default: 30).

    Returns:
        str: Description.

    Raises:
        ValueError: Description.
    """
    if not url.startswith(("http://", "https://")):
        raise ValueError("Invalid URL scheme")

    if timeout <= 0:
        raise ValueError("Timeout must be positive")

    return "response data"


# Test 9: Method with raises
class DataValidator:
    """Summary."""

    def validate(self, data: list[int], min_length: int = 1) -> bool:
        """
        Summary.

        Args:
            data (list[int]): Description.
            min_length (int): Description, optional (default: 1).

        Returns:
            bool: Description.

        Raises:
            ValueError: Description.
        """
        if not data:
            raise ValueError("Data cannot be empty")

        if len(data) < min_length:
            raise ValueError(f"Data must have at least {min_length} items")

        return all(isinstance(x, int) for x in data)


# Test 10: Multiple raises in nested blocks
def complex_validation(value: int, config: Optional[dict] = None) -> tuple[bool, str]:
    """
    Summary.

    Args:
        value (int): Description.
        config (Optional[dict]): Description, optional (default: None).

    Returns:
        tuple[bool, str]: Description.

    Raises:
        ValueError: Description.
    """
    if config is None:
        raise ValueError("Config required")

    if "min" in config:
        if value < config["min"]:
            raise ValueError("Below minimum")

    if "max" in config:
        if value > config["max"]:
            raise ValueError("Above maximum")

    return True, "Valid"


# Test 11: Conditional raise
def process_optional(value: Optional[str]) -> str:
    """
    Summary.

    Args:
        value (Optional[str]): Description.

    Returns:
        str: Description.

    Raises:
        ValueError: Description.
    """
    if value is None:
        raise ValueError("Value cannot be None")
    return value.upper()


# Test 12: Custom exception
class CustomError(Exception):
    """Summary."""

    pass


def raise_custom(x: int) -> None:
    """
    Summary.

    Args:
        x (int): Description.

    Raises:
        CustomError: Description.
    """
    if x == 0:
        raise CustomError("X cannot be zero")


# Test 13: Nested function with raise
def outer_with_validation(threshold: int):
    """
    Summary.

    Args:
        threshold (int): Description.
    """

    def validate_item(item: int) -> bool:
        """
        Summary.

        Args:
            item (int): Description.

        Returns:
            bool: Description.

        Raises:
            ValueError: Description.
        """
        if item < threshold:
            raise ValueError("Item below threshold")
        return True

    return validate_item


# Test 14: Re-raise
def wrapper_function(value: int) -> int:
    """
    Summary.

    Args:
        value (int): Description.

    Returns:
        int: Description.

    Raises:
        ValueError: Description.
    """
    try:
        return int(value)
    except ValueError:
        raise


# Test 15: Raise without parentheses
def simple_raise(flag: bool) -> None:
    """
    Summary.

    Args:
        flag (bool): Description.

    Raises:
        RuntimeError: Description.
    """
    if not flag:
        raise RuntimeError
