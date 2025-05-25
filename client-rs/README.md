

#### Message Flow
1. Load modules (.wasm)
2. Read from stdin
3. Shlex split
4. Pass args to component's handle_command
    a. component handles command
    b. calls message-out export
5. message-out sends bytes to remote
    - figure out how to encode Message -> Protocol
    - Message going to remote will be:
        - Module ID
        - Command ID
        - Transaction ID
        - Command Data
    - Wrapped in Envelope
6. Require response or optional?
    - Require at least confirmation? Probably not necessary
7. Remote sends back response
    - Also includes all info as outgoing message
8. Receive message, send to component's message-in
9. Messages flow as necessary:
    1. new commands run
    2. on-going command data through message-out
    3. on-going command responses through message-in
    - Each component gets a thread?
    - Threads as necessary for each command?