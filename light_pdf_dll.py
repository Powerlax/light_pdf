import ctypes
import sys
import os
from pathlib import Path

class LightPdfDLL:
    def __init__(self, dll_path: str):
        dll_path = Path(dll_path)
        if not dll_path.exists():
            raise FileNotFoundError(f"DLL not found: {dll_path}")
        print(f"Loading DLL from: {dll_path.absolute()}")
        try:
            self.dll = ctypes.CDLL(str(dll_path))
        except OSError as e:
            raise RuntimeError(f"Failed to load DLL: {e}")
        self._setup_functions()
    def _setup_functions(self):
        self.dll.light_pdf_run.argtypes = []
        self.dll.light_pdf_run.restype = ctypes.c_int32
    def run(self) -> int:
        """Run the application. Returns 0 on success."""
        print("Starting Light PDF GUI...")
        try:
            result = self.dll.light_pdf_run()
            return result
        except Exception as e:
            print(f"Error running application: {e}")
            return 1

def main():
    if len(sys.argv) > 1:
        dll_path = sys.argv[1]
    else:
        sys.exit(1)
    try:
        app = LightPdfDLL(dll_path)
        exit_code = app.run()
        if exit_code == 0:
            print("Application closed successfully")
        else:
            print(f"Application exited with code {exit_code}")
        sys.exit(exit_code)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)
    except RuntimeError as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()