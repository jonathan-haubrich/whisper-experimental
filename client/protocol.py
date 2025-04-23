from enum import Enum
from typing import Optional
import struct

class MessageType(Enum):
    LOAD = 0
    COMMAND = 1

MESSAGE_HEADER = '>BQ'

MESSAGE_LOAD_FMT = '>Q{}s'

MESSAGE_COMMAND_FMT = '>QQQ{}s'

MESSAGE_RESPONSE_HEADER = '>Q'
MESSAGE_RESPONSE_BODY_FMT = '{}s'

def pack_message(message_type: MessageType, message: bytes) -> bytes:
    header = struct.pack(MESSAGE_HEADER,
                         message_type.value,
                         len(message))
    
    return header + message

def pack_load(module: bytes) -> bytes:
    module_len = len(module)

    formatted = MESSAGE_LOAD_FMT.format(module_len)

    print(formatted)

    message = struct.pack(formatted, module_len, module)

    return pack_message(MessageType.LOAD, message)

def pack_command(module_id: int, command_id: int, args: Optional[bytes] = b"") -> bytes:
    args_len = len(args)

    formatted = MESSAGE_COMMAND_FMT.format(args_len)

    print("format string:", formatted)

    message = struct.pack(formatted,
                       module_id,
                       command_id,
                       args_len,
                       args)
    
    return pack_message(MessageType.COMMAND, message)

def unpack_response(data: bytes) -> bytes:
    header_len = struct.calcsize(MESSAGE_RESPONSE_HEADER)

    body_len = len(data) - header_len

    formatted = MESSAGE_RESPONSE_HEADER + MESSAGE_RESPONSE_BODY_FMT.format(body_len)

    return struct.unpack(formatted, data)

