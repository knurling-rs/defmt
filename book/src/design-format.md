# Single Format trait

> [!IMPORTANT]
> The design and implementation chapter is outdated

`core::fmt` has several formatting traits, like `Hex` and `Bin`.
These appear as different formatting parameters, like `:x` and `:b`, in format strings and change how integers are formatted: `15` vs `0xF` vs `0b1111`.

`defmt` does not have all these formatting traits.
The rationale is that the device should not make the decision about how an integer is formatted.
The formatting is done in the host so the host should pick the format.
With interactive displays, e.g. web UI, it even becomes possible to change the format on demand, e.g. click the number to change from decimal to hexadecimal representation.
