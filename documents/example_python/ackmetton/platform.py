import sys


# Important method for checking if the system is running this as CPython/MicroPython
def is_micropython() -> bool:
    return hasattr(sys, "implementation") and sys.implementation.name == "micropython"


# The same type of method as above but for checking if it is interpreted Python.
def is_cpython() -> bool:
    return not is_micropython()  # and sys.platform.startswith("linux")
