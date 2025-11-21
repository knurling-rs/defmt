# Preemption

> [!IMPORTANT]
> The design and implementation chapter is outdated

Preemption can also result in re-entrancy.
How to deal with it?
Assuming single-core systems there are two approaches:

1. Disable interrupts in `acquire`; re-enable them in `release`. This means that the logging macros block higher priority interrupts.

2. Have a separate logger per priority level. `acquire` and `release` are now lock-free and don't block interrupts. This requires multiplexing in the transport.
