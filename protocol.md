paperd Protocol
===============

In order to minimize dependencies and overhead, `paperd` uses Unix sockets. The Unix sockets are implemented using only
a few functions, a socket file, and no other dependencies.

Using such an old system does provide a small amount of complexity, though, which is what will be described here. There
are three layers to how we use these message queues, described below.

### The Unix socket

This is not really a layer. Instead, a brief introduction to how Unix sockets work.

Sockets are managed by the kernel, and we retrieve a new Unix socket by calling:

```c
int socket(int domain, int type, int protocol);
```

The `domain` parameter is `AF_UNIX`, and the `type` parameter is `SOCK_STREAM`. This just means to create a Unix socket
in stream mode, you can read more about what that all means in the [man pages](https://man7.org/linux/man-pages/man7/unix.7.html).

Now we have a socket address (that is what the `socket` function returns), we need to bind it to a file on disk so that
other processes can access this socket. We do this with:

```c
int bind(int socket, const struct sockaddr *address, socklen_t address_len);
```

The socket address we got from calling `socket()` above is passed to the `socket` parameter, and the `address` parameter
is just a struct which contains the name (full path) of the socket file to create. 

Now we've created a socket and bound it to a file so clients can access it, we need to listen to that socket for new
connections. We do this with the `listen` function:

```c
int listen(int socket, int backlog);
```

Again, the `socket` parameter is the socket id that we got from `socket()`. The `backlog` parameter is simply the number
of incoming connections that can be queued up before `ECONNREFUSED` is returned. We try to accept new connections as
soon as they come in, but we use `128` to be safe.

Now to wait for a new connection we need to call:

```c
int accept(int socket, struct sockaddr *restrict address, socklen_t *restrict address_len);
```

This returns a new socket descriptor in the `address` struct. The old socket descriptor we created is still listening
for connections, this new socket descriptor can be used to communicate with the client.

We read data from this new socket descriptor with:

```c
ssize_t recv(int socket, void *buffer, size_t length, int flags);
```

This will return up to the number of bytes into `buffer` requested with `length`, unless there is not more data from the
client. The amount of data copied into `buffer` is the return value of this function.

When we're done with the connection we call:

```c
int close(int fildes);
```

With the socket id passed in.

----

In the client we do something similar:

First we create a socket with `socket()`. Then we connect to the server using

```c
int connect(int socket, const struct sockaddr *address, socklen_t address_len);
```

Where we pass our socket id into `socket`, and the socket address for the server's socket into `address`.

Once we have a connection we can send data to the server with:

```c
ssize_t send(int socket, const void *buffer, size_t length, int flags);
```

Which functions the same way as `recv` described above. Note 2 things:

 1. Both the client and the server can call `send` and `recv`, the socket is bi-directional. This is used in `paperd`
    often when the server needs to respond to the client's request.
 1. If the data for a message doesn't fit into a single message, `send` and `recv` will be called in succession until
    all of the data is transferred.

### A message

A complete message is just a complete string of bytes representing a single message. In this context, 'single message'
refers to a single discrete command, rather than a single socket message.

For simplicity, Paper and `paperd` use JSON for passing commands and responses between each other. The JSON data is
encoded using UTF-8 and sent to between the client and server through a series of `send` and `recv` calls with a
buffer size of 1000 bytes.

All messages contain the at least 16 bytes. These 16 bytes represent 2 64-bit integers representing the following 2
fields, in order:

 * `message_type`
 * `message_length`

The fields are sent big endian. The `message_type` determines how the message is parsed. The `message_length` determines
how much data the receiver will expect to receive for a complete message. Note the `message_length` does not include the
first 16 bytes, since that's implicit.

> Note: Several of the messages have a request that is nothing more than `{}`, as the message type is all that needs to
> be known. the reason an empty object is still sent is simply for consistency.

----

### List of messages

#### Protocol Version `0`

Request:
```json
{}
```

Response:
```json
{
  "protocolVersion": 1
}
```

Protocol version is a special case. The "protocol version" is a single integer which specifies the version of the
following messages. This allows updating, adding, reordering, and removing messages below without breaking
compatibility. As long as the protocol version number is bumped accordingly, `paperd` will verify the versions match
before issuing commands to the server.

That being said, the protocol version message `0` _must not change_ else compatibility will be broken. Even between
protocol versions this message must stay the same.

#### Stop `1`

Request:
```json
{}
```
No response.

#### Restart `2`

Request:
```json
{}
```

#### Status `3`

Request:
```json
{}
```

Single Response:
```json
{
  "motd": "<some motd>",
  "serverName": "<some name",
  "serverVersion": "<version>",
  "apiVersion": "<version>",
  "players": ["player1", "player2"],
  "worlds": [
    {
      "name": "world",
      "dimension": "Normal",
      "seed": -4235823458239452,
      "difficulty": "Easy",
      "players": ["player1"],
      "time": "309"
    },
    {
      "name": "world_nether",
      "dimension": "Nether",
      "seed": -4235823458239452,
      "difficulty": "Easy",
      "players": ["player2"],
      "time": "309"
    }
  ],
  "tps": {
    "oneMin": 20.0,
    "fiveMin": 20.0,
    "fifteenMin": 20.0
  },
  "memoryUsage": {
    "usedMemory": "5000 MB",
    "totalMemory": "10000 MB",
    "maxMemory": "10000 MB"
  }
}
```

#### Send Command `4`

Request:
```json
{
  "message": "<some command>"
}
```

No response.

#### Timings `5`

Request:
```json
{}
```

Multiple Responses:
```json
{
  "message": "some message",
  "done": false
}
```

Responses for the timings command will be read until `done` is `true`.

#### Logs Message `6` (for console)

Request:
```json
{
  "pid": 0
}
```

Multiple Responses:
```json
{
  "message": "some message"
}
```

Response for new log messages will read until `End Logs Message` below is received.

#### End Logs Message `7` (for console)

Request:
```json
{
  "pid": 0
}
```

No Response.

#### Console Status Message `8` (for console)

Request:
```json
{}
```

Single Response:
```json
{
  "serverName": "Server Name",
  "players": 0,
  "maxPlayers": 0,
  "tps": 1.0
}
```

#### Tab Complete Message `9` (for console)

Request:
```json
{
  "command": "command string"
}
```

Single Response:
```json
{
  "suggestions": [
    "suggestion 1",
    "suggestion 2"
  ]
}
