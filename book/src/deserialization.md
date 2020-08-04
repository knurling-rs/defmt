# Deserialization

The host has received the log data (binary data).
How to make sense of it?

Let's assume:
- no data loss during transport (reliable transport)
- no interleaving of log frames (no nesting of logging macros)

With these assumptions the decoder can expect the stream of log data to be a series of *log frames*.
