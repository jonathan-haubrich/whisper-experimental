import protocol
import msgpack
import socket

def main():
    module_path = r".\target\debug\module_survey.dll"
    with open(module_path, "rb") as fp:
        module = fp.read()

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM, socket.IPPROTO_TCP)
    s.connect(('172.20.42.169', 4444))

    message = protocol.pack_load(module)

    sent = s.send(message)
    print(f"Sent {sent} bytes")

    response = s.recv(4096)
    print(f"Response: {response}")

    s.close()

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM, socket.IPPROTO_TCP)
    s.connect(('172.20.42.169', 4444))
    s.settimeout(2)

    message = protocol.pack_command(0, 0)

    print(f"Sending: {message}")
    sent = s.send(message)
    print(f"Sent {sent} bytes")

    response = s.recv(4096)

    header, body = protocol.unpack_response(response)

    unpacked = msgpack.unpackb(body)

    print("unpacked:", unpacked)

    print(f"Response: {response}")

    s.close()    

if __name__ == '__main__':
    main()
