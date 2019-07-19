paperd Protocol
===============

In order to minimize dependencies and overhead, `paperd` uses Unix System V IPC message queues. This was chosen in favor
of sockets as an extra file isn't required, and also in favor of communicating through ports, which would require
hosting a server of some sort in Paper (and possibly another in `paperd` as well). The Unix message queues are
implemented using only a few functions which have been around for decades, and no other dependencies.

Using such an old system does provide a small amount of complexity, though, which is what will be described here. There
are three layers to how we use these message queues, described below.

Layer 0
-------
### The message queue

This is not really a layer. Instead, a brief introduction to how Unix System V IPC message queues work.

Message queues are defined with an integer known as an IPC key, or `key_t` in C. This key is unique to the queue, and is
how we send and receive messages to and from this queue. That means the Paper server needs to open a queue with some
key, and `paperd` will need to somehow get the same key to send messages to this queue. This is provided with the
[`ftok`](http://man7.org/linux/man-pages/man3/ftok.3.html) function.

```c
key_t ftok(const char *pathname, int proj_id);
```

The `paper.pid` file that `paperd` creates when starting the Paper server in daemon mode is used as the path name. The
path name argument is the absolute path to this file. The `proj_id` parameter is used to add a tiny bit more randomness
in an attempt to reduce key collisions. This can be any number, as long as both sides are consistent. For us, we use the
`'P'` character, because Paper.

Once we have our key, we can create our queue using the [`msgget`](http://man7.org/linux/man-pages/man2/msgget.2.html)
function.

```c
int msgget(key_t key, int msgflg);
```

Here all we do is pass the key we just got from `ftok` to `msgget` as the first parameter and `0666 | IPC_CREAT` as the
second parameter. This tells `msgget` to create a new queue with `rw-rw-rw-` permissions. The integer returned from
`msgget` is the `msqid`, or message queue id, which we will use for sending messages to and receiving messages from.

Now that we have a queue created, we can send and receive messages on this queue described in Layer 1 below.

Layer 1
-------
### A single message

A message is sent with the [`msgsnd`](http://man7.org/linux/man-pages/man2/msgsnd.2.html) function.

```c
int msgsnd(int msqid, const void *msgp, size_t msgsz, int msgflg);
```

`msgflg` is optional and we always pass `0`.

The first argument is just the queue id, which we got from `msgget` above. The second is a pointer to our message
struct, which looks like this:

```c
#declare MESSAGE_LENGTH 100

struct message {
    long m_type;
    data struct message_data;
};

struct message_data {
    int32_t response_chan;
    uint32_t response_pid;
    int16_t message_type;
    uint8_t message_length;
    uint8_t[MESSAGE_LENGTH] message;
};
```

The struct can't contain any pointers since when the message is received they won't have access to any of that memory,
so we can't use any VLAs. This is why Layer 2 below is necessary, but we'll get to that later.

When we send a receive messages we need to use the same message type (`m_type` above) on both both sides. For Paper we
use the integer `0x7654` for all messages.

The `data` field in the `message` struct contains all of our own data. The `m_type` field is required to be exactly the
length of a `long` on the current system, everything past that we will specify with the `size_t msgsz` parameter of the
call to `msgsnd`. That is, the call will contain a pointer to our message and `sizeof(struct message_data)`. The
underlying IPC system won't do anything more than just copy the length of data we provided into the message, and copy it
back out when we receive.

Our `message_data` has the following fields:
 * `resopnse_chan`: The `msqid` of the IPC message queue to send responses to. `paperd` will create its own message
                    channel and pass the ID in this part of the message so the Paper server can send messages back to
                    `paperd`.
 * `response_pid`: The PID of the current running `paperd` process. This is used so Paper can check to make sure the
                   `paperd` process is still alive if it hasn't received a message in a while.
 * `message_type`: This defines the message type used for Layer 3. This determines the different kinds of messages Paper
                   and `paperd` will use.
 * `message_lenth`: The length of the `message` field that is actually used for this message. `message` is a
                    fixed-length array of 100 bytes, but not every byte may be used in a message.
 * `message`: A fixed-length array of 100 bytes used to store part of the Layer 2 command. `message_length` determines
              how many of the bytes in this field are actually used. The bytes after `message_length` are not part of
              the message and may contain anything.

The data stored in `message` is raw byte data at this level. Layer 3 will give that data meaning.

For receiving a message the [`msgrcv`](http://man7.org/linux/man-pages/man3/msgrcv.3p.html) function is used.

```c
ssize_t msgrcv(int msqid, void *msgp, size_t msgsz, long msgtyp, int msgflg);
```

`msqid`, `msgp`, `msgsz`, `msgtyp`, and `msgflg` are all the same as when we called `msgsnd`, just this time the message
struct we passed will be set with the data from the message we are receiving.

Layer 2
-------
### Buffered messages

Layer 2 simply buffers messages together until a complete message is formed. Messages are kept fairly short at just over
100 bytes due to message size limitations (which actually can't be changed on macOS). A single Layer 3 document is
created to be sent as a message, and that document is passed simply as an array of bytes to Layer 2. Layer 2 then chunks
the data into 100-byte increments until all of the data is completely sent.

Layer 1 messages are received in order and appended to a byte buffer until the final message is sent. Once the final
message is sent the byte buffer can be parsed as a complete message.

A Layer 1 message is marked as the final message in a command when the first bit in `message_length` is set to 1.
`message_length` is an 8-bit unsigned integer, but it only needs to count up to 100 at most. The bottom 7 bits will
count up to 128 which is plenty. The first bit is instead reserved for marking a message as final.

When the first bit in `message_length` is set to 1 then the last message is appended to the byte buffer and passed to
Layer 3 to be parsed as a complete document.

Layer 3
-------
### A document

A complete document is just a complete string of bytes representing a single message. In this context, 'single message'
refers to a single discrete command, rather than an IPC message queue message.

For simplicity, Paper and `paperd` use JSON for passing commands and responses between each other. The JSON data is
encoded using UTF-8 and sent to Layer 2 to be sent as a series of smaller messages.

Once a full command has been received, the `message_type` field of the last message is used to determine how to parse
the binary data as a command. The following message types are available:

> Note: Several of the messages have a request that is nothing more than `{}`, as the message type is all that needs to
> be known. the reason an empty object is still sent is simply for consistency.

----

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
