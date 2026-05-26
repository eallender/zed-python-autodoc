"""Example Python file to demonstrate python-autodoc LSP capabilities.

All doc-strings were generated with python-autodoc
"""


# Example 1: Simple function with type hints
def greet(name: str, greeting: str = "Hello") -> str:
    """
    Summary.

    Args:
        name (str): Description.
        greeting (str): Description, optional (default: "Hello").

    Returns:
        str: Description.
    """
    pass


# Example 2: Function without type annotations
def add(a, b):
    """
    Summary.

    Args:
        a (type): Description.
        b (type): Description.
    """
    pass


# Example 3: Function with multiple parameters and defaults
def create_user(username: str, email: str, age: int = 18, active: bool = True) -> dict:
    """
    Summary.

    Args:
        username (str): Description.
        email (str): Description.
        age (int): Description, optional (default: 18).
        active (bool): Description, optional (default: True).

    Returns:
        dict: Description.
    """
    pass


# Example 4: Async function
async def fetch_data(url: str, timeout: int = 30) -> dict:
    """
    Summary.

    Args:
        url (str): Description.
        timeout (int): Description, optional (default: 30).

    Returns:
        dict: Description.
    """
    pass


# Example 5: Function with *args and **kwargs
def process_data(*args, **kwargs):
    """
    Summary.

    Args:
        *args (type): Description.
        **kwargs (type): Description.
    """
    pass


# Example 6: Multi-line function signature
def complex_calculation(
    value: float,
    multiplier: int,
    offset: float = 0.0,
    precision: int = 2,
) -> float:
    """
    Summary.

    Args:
        value (float): Description.
        multiplier (int): Description.
        offset (float): Description, optional (default: 0.0).
        precision (int): Description, optional (default: 2).

    Returns:
        float: Description.
    """
    pass


# Example 7: Class without __init__
class SimpleLogger:
    """Summary."""

    def log(self, message: str) -> None:
        """
        Summary.

        Args:
            message (str): Description.
        """
        pass


# Example 8: Class with __init__ (PEP 257: document __init__ params in __init__, not class)
class Point:
    """Summary."""

    def __init__(self, x: float, y: float):
        """
        Summary.

        Args:
            x (float): Description.
            y (float): Description.
        """
        self.x = x
        self.y = y

    def distance_from_origin(self) -> float:
        """
        Summary.

        Returns:
            float: Description.
        """
        pass


# Example 9: Class with complex __init__
class DatabaseConnection:
    """Summary."""

    def __init__(
        self,
        host: str,
        port: int,
        username: str,
        password: str,
        database: str = "default",
        timeout: int = 30,
    ):
        """
        Summary.

        Args:
            host (str): Description.
            port (int): Description.
            username (str): Description.
            password (str): Description.
            database (str): Description, optional (default: "default").
            timeout (int): Description, optional (default: 30).
        """
        self.host = host
        self.port = port
        self.username = username
        self.password = password
        self.database = database
        self.timeout = timeout

    def connect(self) -> bool:
        """
        Summary.

        Returns:
            bool: Description.
        """
        pass


# Example 10: __init__ with default values and optional parameters
class UserProfile:
    """Summary."""

    def __init__(
        self,
        user_id: int,
        name: str,
        email: str = None,
        bio: str = "",
        verified: bool = False,
    ):
        """
        Summary.

        Args:
            user_id (int): Description.
            name (str): Description.
            email (str): Description, optional (default: None).
            bio (str): Description, optional (default: "").
            verified (bool): Description, optional (default: False).
        """
        self.user_id = user_id
        self.name = name
        self.email = email
        self.bio = bio
        self.verified = verified


# Example 11: Method that returns None (Returns section will be omitted)
def reset_counter(counter: dict) -> None:
    """
    Summary.

    Args:
        counter (dict): Description.
    """
    pass


# Example 12: Function with complex type hints
def merge_configs(configs: list[dict], override: dict = None) -> dict:
    """
    Summary.

    Args:
        configs (list[dict]): Description.
        override (dict): Description, optional (default: None).

    Returns:
        dict: Description.
    """
    pass


# Example 13: Nested class
class Outer:
    """Summary."""

    class Inner:
        """Summary."""

        def __init__(self, value: int):
            """
            Summary.

            Args:
                value (int): Description.
            """
            self.value = value


# Example 14: Static method and class method
class MathUtils:
    """Summary."""

    @staticmethod
    def add(a: int, b: int) -> int:
        """
        Summary.

        Args:
            a (int): Description.
            b (int): Description.

        Returns:
            int: Description.
        """
        pass

    @classmethod
    def create_from_string(cls, value: str):
        """
        Summary.

        Args:
            value (str): Description.
        """
        pass


# Example 15: Property with type hints
class Temperature:
    """Summary."""

    def __init__(self, celsius: float):
        """
        Summary.

        Args:
            celsius (float): Description.
        """
        self._celsius = celsius

    @property
    def fahrenheit(self) -> float:
        """
        Summary.

        Returns:
            float: Description.
        """
        return (self._celsius * 9 / 5) + 32
