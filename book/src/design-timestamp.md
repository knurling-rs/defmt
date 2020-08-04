# Timestamp

In the current implementation timestamps are absolute (time elapsed since the start of the program) and in microseconds.
Timestamps are LEB128 encoded before serialization.

> TODO we may want to consider using delta encoding in the future
