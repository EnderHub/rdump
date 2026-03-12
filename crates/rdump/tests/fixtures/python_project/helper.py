# FIXME: Hardcoded path
import os

class Helper:
    def __init__(self):
        self.path = "/tmp/data"
        self.do_setup()

    def do_setup(self):
        print("Setup complete")

def run_helper():
    h = Helper()
    return h.path

if __name__ == "__main__":
    run_helper()
