# Acquire-release

One solution to the re-entrancy issue that's deadlock-free is to make the log macros *take* the logger and hold it until it's done with it.
In case of nesting any inner take attempt will silently fail.

So the log macros may expand to something like this:
(let's ignore data races / race conditions for now)

``` rust
if let Some(logger) = Logger::acquire() {
    logger.serialize_interned_string_and_etc();
    release(logger); // <- logger can be acquired again after this
} else {
    // silent failure: do nothing here
}
```

This means that invoking logging macros from `Format` implementations will silently fail.
But note that allowing such operation would result in interleaving of log frames.
To a decoder/parser interleaved log frames are the same as corrupted log frames.
So we actually want to forbid this operation.
