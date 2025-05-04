import argparse
import cmd
import os
import shlex

import client as c

class CmdLine(cmd.Cmd):
    def __init__(self, client: c.Client):
        self.client = client
        super().__init__()

    def do_load(self, arg):
        parser = argparse.ArgumentParser(prog="load",
            description="Load a module",
            exit_on_error=False,
            add_help=False)
        parser.add_argument("-n", "--name", help="Module name", required=True)
        split = shlex.split(arg)
        try:
            args = parser.parse_args(split)
        except argparse.ArgumentError as ae:
            print(ae)
            return

        module_path = f'{c.MODULES_DIR}\\{args.name}\\module.dll'
        if not os.path.exists(module_path):
            print(f"Module [{args.name}] does not exist")
            return
        
        with open(module_path, 'rb') as fp:
            module = fp.read()

        module_id = self.client.load(module)
        print(f"Module loaded successfully, assigned id {module_id}")

    def do_modules(self, arg):
        parser = argparse.ArgumentParser(prog="modules",
            description="List available and loaded modules",
            exit_on_error=False,
            add_help=False)
        split = shlex.split(arg)
        try:
            args = parser.parse_args(split)
        except argparse.ArgumentError as ae:
            print(ae)
            return
        
        loaded = self.client.loaded()
        available = os.listdir(c.MODULES_DIR)

        print("Loaded:")
        for module in loaded:
            print(f"  {module}")

        print("Available:")
        for module in available:
            if module not in loaded:
                print(f"  {module}")