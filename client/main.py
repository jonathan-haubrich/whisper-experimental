from client import Client
from cmdline import CmdLine

def main():
    client = Client()
    client.connect(('172.20.42.192', 4444))

    cmdline = CmdLine(client)
    cmdline.cmdloop()
    
    client.close()

if __name__ == '__main__':
    main()