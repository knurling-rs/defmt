SECTIONS
{
  .binfmt (INFO) :
  {
    /* Format implementations for primitives like u8 */
    *(.binfmt.prim.*);

    /* ERROR logging level */
    _binfmt_error_start = .;
    *(.binfmt.error.*);
    _binfmt_error_end = .;

    /* WARN logging level */
    _binfmt_warn_start = .;
    *(.binfmt.warn.*);
    _binfmt_warn_end = .;

    /* INFO logging level */
    _binfmt_info_start = .;
    *(.binfmt.info.*);
    _binfmt_info_end = .;

    /* DEBUG logging level */
    _binfmt_debug_start = .;
    *(.binfmt.debug.*);
    _binfmt_debug_end = .;

    /* TRACE logging level */
    _binfmt_trace_start = .;
    *(.binfmt.trace.*);
    _binfmt_trace_end = .;

    /* Format/write! strings */
    *(.binfmt.fmt.*);

    /* User interned strings (Str) */
    *(.binfmt.str.*);

    _binfmt_version_ = 1;
  }
}

ASSERT(SIZEOF(.binfmt) < 16384, ".binfmt section cannot contain more than (1<<14) interned strings");
