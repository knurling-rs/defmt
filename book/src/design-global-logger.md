# Global logger

> ⚠️ The design and implementation chapter is outdated ⚠️

The global logger needs to operate correctly (be memory safe and not interleave log data) in presence of race conditions and re-entrant invocations.
Race conditions can be avoided with mutexes but re-entrancy can occur even if mutexes are used and shouldn't result in deadlocks.
