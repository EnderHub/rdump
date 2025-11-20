# Test file for rdump
class MyClass:
    def __init__(self, value):
        self.value = value

    def my_method(self):
        """A docstring."""
        print(f"Value: {self.value}")

# A comment

def do_setup():
    return "setup"

def run_helper():
    return do_setup()
