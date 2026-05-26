"""Example file demonstrating complex type annotation support."""

from collections.abc import Sequence
from typing import (
    Annotated,
    Any,
    Callable,
    Dict,
    Final,
    Generic,
    List,
    Literal,
    Optional,
    TypeVar,
    Union,
)


# Test 1: Union types (old style)
def process_value_union(
    value: Union[str, int], default: Union[str, int] = 0
) -> Union[str, int]:
    """
    Summary.

    Args:
        value (Union[str, int]): Description.
        default (Union[str, int]): Description, optional (default: 0).

    Returns:
        Union[str, int]: Description.
    """
    pass


# Test 2: Union types (Python 3.10+ pipe syntax)
def process_value_pipe(value: str | int, default: str | int = 0) -> str | int:
    """
    Summary.

    Args:
        value (str | int): Description.
        default (str | int): Description, optional (default: 0).

    Returns:
        str | int: Description.
    """
    pass


# Test 3: Optional types
def find_user(user_id: int, cache: Optional[dict] = None) -> Optional[str]:
    """
    Summary.

    Args:
        user_id (int): Description.
        cache (Optional[dict]): Description, optional (default: None).

    Returns:
        Optional[str]: Description.
    """
    pass


# Test 4: Callable types
def register_callback(
    callback: Callable[[int, str], bool],
    fallback: Callable[[str], None] = None,
) -> None:
    """
    Summary.

    Args:
        callback (Callable[[int, str], bool]): Description.
        fallback (Callable[[str], None]): Description, optional (default: None).
    """
    pass


# Test 5: Nested generic types
def merge_data(
    data: list[dict[str, Any]],
    overrides: dict[str, list[int]] = None,
) -> list[dict[str, Any]]:
    """
    Summary.

    Args:
        data (list[dict[str, Any]]): Description.
        overrides (dict[str, list[int]]): Description, optional (default: None).

    Returns:
        list[dict[str, Any]]: Description.
    """
    pass


# Test 6: TypeVar and Generic types
T = TypeVar("T")


def get_first(items: List[T], default: T = None) -> T:
    """
    Summary.

    Args:
        items (List[T]): Description.
        default (T): Description, optional (default: None).

    Returns:
        T: Description.
    """
    pass


# Test 7: Multiple TypeVars
K = TypeVar("K")
V = TypeVar("V")


def transform_dict(data: Dict[K, V], transformer: Callable[[V], V]) -> Dict[K, V]:
    """
    Summary.

    Args:
        data (Dict[K, V]): Description.
        transformer (Callable[[V], V]): Description.

    Returns:
        Dict[K, V]: Description.
    """
    pass


# Test 8: Very long type annotation (tests wrapping behavior)
def complex_processor(
    data: dict[str, Union[int, str, list[dict[str, Any]]]],
    validators: list[Callable[[dict[str, Any]], bool]],
    transformers: dict[str, Callable[[Union[int, str]], str]] = None,
    config: Optional[dict[str, Union[str, int, bool, list[str]]]] = None,
) -> tuple[dict[str, Any], list[str]]:
    """
    Summary.

    Args:
        data (dict[str, Union[int, str, list[dict[str, Any]]]]): Description.
        validators (list[Callable[[dict[str, Any]], bool]]): Description.
        transformers (dict[str, Callable[[Union[int, str]], str]]): Description, optional (default: None).
        config (Optional[dict[str, Union[str, int, bool, list[str]]]]): Description, optional (default: None).

    Returns:
        tuple[dict[str, Any], list[str]]: Description.
    """
    pass


# Test 9: Nested Callables
def higher_order_function(
    func: Callable[[Callable[[int], str]], list[str]],
) -> Callable[[int], str]:
    """
    Summary.

    Args:
        func (Callable[[Callable[[int], str]], list[str]]): Description.

    Returns:
        Callable[[int], str]: Description.
    """
    pass


# Test 10: Union with many types
def multi_type_handler(
    value: Union[str, int, float, bool, list, dict, None],
) -> Union[str, int, float, bool, list, dict, None]:
    """
    Summary.

    Args:
        value (Union[str, int, float, bool, list, dict, None]): Description.

    Returns:
        Union[str, int, float, bool, list, dict, None]: Description.
    """
    pass


# Test 11: Generic class
class Container(Generic[T]):
    """Summary."""

    def __init__(self, value: T):
        """
        Summary.

        Args:
            value (T): Description.
        """
        self.value = value

    def get(self) -> T:
        """
        Summary.

        Returns:
            T: Description.
        """
        pass

    def set(self, value: T) -> None:
        """
        Summary.

        Args:
            value (T): Description.
        """
        pass


# Test 12: Multiple inheritance with generics
class KeyValueStore(Generic[K, V]):
    """Summary."""

    def __init__(self, initial_data: Dict[K, V] = None):
        """
        Summary.

        Args:
            initial_data (Dict[K, V]): Description, optional (default: None).
        """
        self.data = initial_data or {}

    def get(self, key: K, default: V = None) -> Optional[V]:
        """
        Summary.

        Args:
            key (K): Description.
            default (V): Description, optional (default: None).

        Returns:
            Optional[V]: Description.
        """
        pass

    def set(self, key: K, value: V) -> None:
        """
        Summary.

        Args:
            key (K): Description.
            value (V): Description.
        """
        pass


# Test 13: Complex nested Optional and Union
def parse_config(
    config: Optional[Union[str, dict[str, Union[str, int, list[str]]]]] = None,
) -> dict[str, Any]:
    """
    Summary.

    Args:
        config (Optional[Union[str, dict[str, Union[str, int, list[str]]]]]): Description, optional (default: None).

    Returns:
        dict[str, Any]: Description.
    """
    pass


# Test 14: Sequence and other abstract types
def process_sequence(
    items: Sequence[Union[int, str]],
    filter_func: Optional[Callable[[Union[int, str]], bool]] = None,
) -> list[Union[int, str]]:
    """
    Summary.

    Args:
        items (Sequence[Union[int, str]]): Description.
        filter_func (Optional[Callable[[Union[int, str]], bool]]): Description, optional (default: None).

    Returns:
        list[Union[int, str]]: Description.
    """
    pass


# Test 15: Extremely long single-line type
def extremely_long_type_annotation(
    value: dict[
        str,
        Union[
            list[dict[str, Union[int, str, float, bool]]],
            Callable[[dict[str, Any]], Optional[Union[str, int]]],
        ],
    ] = None,
) -> Optional[Union[dict[str, list[Union[int, str]]], list[dict[str, Any]]]]:
    """
    Summary.

    Args:
        value (dict[
            str,
            Union[
                list[dict[str, Union[int, str, float, bool]]],
                Callable[[dict[str, Any]], Optional[Union[str, int]]],
            ],
        ]): Description, optional (default: None).

    Returns:
        Optional[Union[dict[str, list[Union[int, str]]], list[dict[str, Any]]]]: Description.
    """
    pass


# Test 16: Literal types (Python 3.8+)
def set_mode(mode: Literal["read", "write", "append"]) -> None:
    """
    Summary.

    Args:
        mode (Literal["read", "write", "append"]): Description.
    """
    pass


# Test 17: Final types
def process_constant(value: Final[int]) -> None:
    """
    Summary.

    Args:
        value (Final[int]): Description.
    """
    pass


# Test 18: Annotated types (Python 3.9+)
def validate_positive(value: Annotated[int, "must be positive"]) -> bool:
    """
    Summary.

    Args:
        value (Annotated[int, "must be positive"]): Description.

    Returns:
        bool: Description.
    """
    pass


# Test 19: Combining multiple complex patterns
def complex_api_handler(
    endpoint: Literal["users", "posts", "comments"],
    filters: Optional[dict[str, Union[str, int, list[str]]]] = None,
    transform: Callable[[dict[str, Any]], dict[str, Any]] = None,
) -> Union[list[dict[str, Any]], dict[str, str]]:
    """
    Summary.

    Args:
        endpoint (Literal["users", "posts", "comments"]): Description.
        filters (Optional[dict[str, Union[str, int, list[str]]]]): Description, optional (default: None).
        transform (Callable[[dict[str, Any]], dict[str, Any]]): Description, optional (default: None).

    Returns:
        Union[list[dict[str, Any]], dict[str, str]]: Description.
    """
    pass


# Test 20: Nested generic class with methods
class Repository(Generic[T]):
    """Summary."""

    def find_by_id(self, id: int) -> Optional[T]:
        """
        Summary.

        Args:
            id (int): Description.

        Returns:
            Optional[T]: Description.
        """
        pass

    def find_all(self, filters: dict[str, Any] = None) -> List[T]:
        """
        Summary.

        Args:
            filters (dict[str, Any]): Description, optional (default: None).

        Returns:
            List[T]: Description.
        """
        pass

    def save(self, entity: T) -> T:
        """
        Summary.

        Args:
            entity (T): Description.

        Returns:
            T: Description.
        """
        pass

    def delete(self, entity: T) -> bool:
        """
        Summary.

        Args:
            entity (T): Description.

        Returns:
            bool: Description.
        """
        pass
