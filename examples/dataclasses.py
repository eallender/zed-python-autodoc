"""Example file demonstrating dataclass docstring support.

This file contains dataclass examples to test the python-autodoc extension's
handling of @dataclass decorator and Attributes section generation.

To test: Position cursor after the colon on any class definition and type
"""

from dataclasses import dataclass, field
from typing import Any, Callable, ClassVar, InitVar, List, Optional


# Test 1: Basic dataclass
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


# Test 2: Dataclass with defaults
@dataclass
class Person:
    """
    Summary.

    Attributes:
        name (str): Description.
        age (int): Description.
        email (str): Description, optional (default: "unknown@example.com").
    """

    name: str
    age: int
    email: str = "unknown@example.com"


# Test 3: Dataclass with Optional fields
@dataclass
class User:
    """
    Summary.

    Attributes:
        username (str): Description.
        email (Optional[str]): Description, optional (default: None).
        bio (Optional[str]): Description, optional (default: None).
        verified (bool): Description, optional (default: False).
    """

    username: str
    email: Optional[str] = None
    bio: Optional[str] = None
    verified: bool = False


# Test 4: Dataclass with ClassVar (should be skipped)
@dataclass
class Counter:
    """
    Summary.

    Attributes:
        value (int): Description.
    """

    value: int
    total: ClassVar[int] = 0  # Should NOT appear in Attributes


# Test 5: Dataclass with InitVar (should be skipped)
@dataclass
class Item:
    """
    Summary.

    Attributes:
        name (str): Description.
        price (float): Description.
    """

    name: str
    price: float
    database_id: InitVar[int] = None  # Should NOT appear in Attributes


# Test 6: Dataclass with complex types
@dataclass
class Config:
    """
    Summary.

    Attributes:
        data (list[dict[str, Any]]): Description.
        callbacks (List[Callable[[int], str]]): Description.
        options (Optional[dict[str, Union[str, int]]]): Description, optional (default: None).
    """

    data: list[dict[str, Any]]
    callbacks: List[Callable[[int], str]]
    options: Optional[dict[str, Union[str, int]]] = None


# Test 7: Dataclass with field() defaults
@dataclass
class Container:
    """
    Summary.

    Attributes:
        items (List[str]): Description, optional (default: field(default_factory=list)).
        metadata (dict): Description, optional (default: field(default_factory=dict)).
    """

    items: List[str] = field(default_factory=list)
    metadata: dict = field(default_factory=dict)


# Test 8: Frozen dataclass
@dataclass(frozen=True)
class Coordinate:
    """
    Summary.

    Attributes:
        latitude (float): Description.
        longitude (float): Description.
        altitude (float): Description, optional (default: 0.0).
    """

    latitude: float
    longitude: float
    altitude: float = 0.0


# Test 9: Dataclass with post_init
@dataclass
class Rectangle:
    """
    Summary.

    Attributes:
        width (float): Description.
        height (float): Description.
    """

    width: float
    height: float
    database: InitVar[Any] = None

    def __post_init__(self, database):
        pass


# Test 10: Nested dataclass
@dataclass
class Address:
    """
    Summary.

    Attributes:
        street (str): Description.
        city (str): Description.
        zipcode (str): Description.
    """

    street: str
    city: str
    zipcode: str


@dataclass
class Customer:
    """
    Summary.

    Attributes:
        name (str): Description.
        address (Address): Description.
        phone (Optional[str]): Description, optional (default: None).
    """

    name: str
    address: Address
    phone: Optional[str] = None


# Test 11: Dataclass with Union types
@dataclass
class Response:
    """
    Summary.

    Attributes:
        status (str | int): Description.
        data (str | dict | None): Description, optional (default: None).
    """

    status: str | int
    data: str | dict | None = None


# Test 12: Dataclass using dataclasses.dataclass
import dataclasses


@dataclasses.dataclass
class Book:
    """
    Summary.

    Attributes:
        title (str): Description.
        author (str): Description.
        isbn (str): Description.
        pages (int): Description, optional (default: 0).
    """

    title: str
    author: str
    isbn: str
    pages: int = 0


# Test 13: Dataclass with ordering
@dataclass(order=True)
class Priority:
    """
    Summary.

    Attributes:
        level (int): Description.
        name (str): Description.
    """

    level: int
    name: str


# Test 14: Dataclass with slots
@dataclass(slots=True)
class Vector:
    """
    Summary.

    Attributes:
        x (float): Description.
        y (float): Description.
        z (float): Description, optional (default: 0.0).
    """

    x: float
    y: float
    z: float = 0.0


# Test 15: Complex dataclass with multiple features
@dataclass
class TaskConfig:
    """
    Summary.

    Attributes:
        name (str): Description.
        priority (int): Description.
        tags (List[str]): Description, optional (default: field(default_factory=list)).
        metadata (dict[str, Any]): Description, optional (default: field(default_factory=dict)).
        callback (Optional[Callable[[str], None]]): Description, optional (default: None).
        retry_count (int): Description, optional (default: 3).
        timeout (float): Description, optional (default: 30.0).
    """

    name: str
    priority: int
    tags: List[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)
    callback: Optional[Callable[[str], None]] = None
    retry_count: int = 3
    timeout: float = 30.0
    _internal_id: InitVar[int] = None
    _cache: ClassVar[dict] = {}


# Test 16: Regular class (should NOT get Attributes)
class RegularClass:
    """Summary."""

    x: float
    y: float


# Test 17: Dataclass with property-like annotations
@dataclass
class Product:
    """
    Summary.

    Attributes:
        name (str): Description.
        price (float): Description.
        discount (float): Description, optional (default: 0.0).
    """

    name: str
    price: float
    discount: float = 0.0

    @property
    def final_price(self) -> float:
        """
        Summary.

        Returns:
            float: Description.
        """
        return self.price * (1 - self.discount)


# Test 18: Dataclass with validators
@dataclass
class ValidatedUser:
    """
    Summary.

    Attributes:
        username (str): Description.
        age (int): Description.
        email (Optional[str]): Description, optional (default: None).
    """

    username: str
    age: int
    email: Optional[str] = None

    def __post_init__(self):
        """
        Summary.

        Raises:
            ValueError: Description.
        """
        if self.age < 0:
            raise ValueError("Age cannot be negative")
