import protocol
import msgpack
import socket
import struct
from typing import Tuple

MODULES_DIR = "./modules"

class Client:
    def __init__(self):
        self.socket = None

    def connect(self, endpoint):
        if self.socket is not None:
            raise Exception("Socket already connected")
        
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM, socket.IPPROTO_TCP)
        self.socket.connect(endpoint)

    def close(self):
        if self.socket is None:
            return

        self.socket.shutdown(socket.SHUT_RDWR)
        self.socket.close()

    def load(self, module) -> int:
        message = protocol.pack_load(module)
        self.socket.sendall(message)

        response = self.recv_message()

        module_id = protocol.unpack_response(response)

        return module_id

    def recv_message(self) -> bytes:
        header, message_len = self.recv_message_len()

        message = self.recv_all(message_len)

        return header + message

    def recv_message_len(self) -> Tuple[bytes, int]:
        response_len_size = struct.calcsize(protocol.MESSAGE_RESPONSE_HEADER)

        header_len_bytes = self.recv_all(response_len_size)

        unpacked = struct.unpack(protocol.MESSAGE_RESPONSE_HEADER, header_len_bytes)

        if len(unpacked) < 1:
            raise Exception("Failed to unpack message response header")
        
        return header_len_bytes, unpacked[0]
    
    def recv_all(self, requested) -> bytes:
        received = 0
        data = b''

        while received < requested:
            chunk = self.socket.recv(requested - received)
            if chunk is None:
                break
            received += len(chunk)
            data += chunk

        return data
