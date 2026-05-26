"""Example file demonstrating nested function support."""

from typing import Any, Callable, Optional


# Test 1: Basic nested function
def outer():
    """Summary."""

    def inner(x: int) -> str:
        """
        Summary.

        Args:
            x (int): Description.

        Returns:
            str: Description.
        """
        pass


# Test 2: Nested function with outer params
def create_multiplier(factor: int) -> Callable[[int], int]:
    """
    Summary.

    Args:
        factor (int): Description.

    Returns:
        Callable[[int], int]: Description.
    """

    def multiply(value: int) -> int:
        """
        Summary.

        Args:
            value (int): Description.

        Returns:
            int: Description.
        """
        return value * factor

    return multiply


# Test 3: Deeply nested functions (3 levels)
def level1(a: str):
    """
    Summary.

    Args:
        a (str): Description.
    """

    def level2(b: int):
        """
        Summary.

        Args:
            b (int): Description.
        """

        def level3(c: float) -> dict:
            """
            Summary.

            Args:
                c (float): Description.

            Returns:
                dict: Description.
            """
            pass

        return level3

    return level2


# Test 4: Nested function with multiline signature
def processor():
    """Summary."""

    def process_data(
        data: list[dict[str, Any]],
        validate: bool = True,
        transform: Optional[Callable[[dict], dict]] = None,
    ) -> list[dict[str, Any]]:
        """
        Summary.

        Args:
            data (list[dict[str, Any]]): Description.
            validate (bool): Description, optional (default: True).
            transform (Optional[Callable[[dict], dict]]): Description, optional (default: None).

        Returns:
            list[dict[str, Any]]: Description.
        """
        pass


# Test 5: Nested async function
def create_async_fetcher(base_url: str):
    """
    Summary.

    Args:
        base_url (str): Description.
    """

    async def fetch(endpoint: str, params: dict = None) -> dict:
        """
        Summary.

        Args:
            endpoint (str): Description.
            params (dict): Description, optional (default: None).

        Returns:
            dict: Description.
        """
        pass

    return fetch


# Test 6: Nested function with complex types
def create_handler():
    """Summary."""

    def handle(
        data: dict[str, list[int]],
        callback: Callable[[list[int]], Optional[str]],
    ) -> Optional[str]:
        """
        Summary.

        Args:
            data (dict[str, list[int]]): Description.
            callback (Callable[[list[int]], Optional[str]]): Description.

        Returns:
            Optional[str]: Description.
        """
        pass


# Test 7: Multiple nested functions at same level
def outer_with_multiple():
    """Summary."""

    def helper1(x: int) -> int:
        """
        Summary.

        Args:
            x (int): Description.

        Returns:
            int: Description.
        """
        pass

    def helper2(y: str) -> str:
        """
        Summary.

        Args:
            y (str): Description.

        Returns:
            str: Description.
        """
        pass

    def main_logic(a: float, b: float) -> tuple[float, float]:
        """
        Summary.

        Args:
            a (float): Description.
            b (float): Description.

        Returns:
            tuple[float, float]: Description.
        """
        pass


# Test 8: Nested function with *args and **kwargs
def decorator_factory(prefix: str):
    """
    Summary.

    Args:
        prefix (str): Description.
    """

    def decorator(func: Callable) -> Callable:
        """
        Summary.

        Args:
            func (Callable): Description.

        Returns:
            Callable: Description.
        """

        def wrapper(*args, **kwargs):
            """
            Summary.

            Args:
                *args (type): Description.
                **kwargs (type): Description.
            """
            pass

        return wrapper

    return decorator


# Test 9: Nested function inside class method
class DataProcessor:
    """Summary."""

    def process(self, items: list[Any]) -> list[Any]:
        """
        Summary.

        Args:
            items (list[Any]): Description.

        Returns:
            list[Any]: Description.
        """

        def filter_item(item: Any) -> bool:
            """
            Summary.

            Args:
                item (Any): Description.

            Returns:
                bool: Description.
            """
            pass

        def transform_item(item: Any) -> Any:
            """
            Summary.

            Args:
                item (Any): Description.

            Returns:
                Any: Description.
            """
            pass

        return [transform_item(item) for item in items if filter_item(item)]


# Test 10: Closure with type annotations
def create_counter(start: int = 0) -> Callable[[], int]:
    """
    Summary.

    Args:
        start (int): Description, optional (default: 0).

    Returns:
        Callable[[], int]: Description.
    """
    count = start

    def increment() -> int:
        """
        Summary.

        Returns:
            int: Description.
        """
        nonlocal count
        count += 1
        return count

    return increment


# Test 11: Generator nested function
def create_generator():
    """Summary."""

    def generate_sequence(start: int, end: int, step: int = 1):
        """
        Summary.

        Args:
            start (int): Description.
            end (int): Description.
            step (int): Description, optional (default: 1).
        """
        pass


# Test 12: Nested function with Union types
def create_validator():
    """Summary."""

    def validate(value: str | int | float) -> bool:
        """
        Summary.

        Args:
            value (str | int | float): Description.

        Returns:
            bool: Description.
        """
        pass


# Test 13: Nested lambda (shouldn't generate docstring, but outer should work)
def with_lambda(x: int) -> Callable[[int], int]:
    """
    Summary.

    Args:
        x (int): Description.

    Returns:
        Callable[[int], int]: Description.
    """
    return lambda y: x + y


# Test 14: Nested function with decorators
def outer_decorator():
    """Summary."""

    @staticmethod
    def static_inner(value: str) -> str:
        """
        Summary.

        Args:
            value (str): Description.

        Returns:
            str: Description.
        """
        pass


# Test 15: Nested function returning Optional
def finder():
    """Summary."""

    def find_by_id(id: int, data: list[dict]) -> Optional[dict]:
        """
        Summary.

        Args:
            id (int): Description.
            data (list[dict]): Description.

        Returns:
            Optional[dict]: Description.
        """
        pass
