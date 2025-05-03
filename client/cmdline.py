import argparse
import cmd
import os
import shlex

from client import Client

class CmdLine(cmd.Cmd):
    def __init__(self, client: Client):
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

        module_path = f'modules\\{args.name}\\module.dll'
        if not os.path.exists(module_path):
            print(f"Module [{args.name}] does not exist")
            return
        
        with open(module_path, 'rb') as fp:
            module = fp.read()

        module_id = self.client.load(module)
        print(f"Module loaded successfully, assigned id {module_id}")