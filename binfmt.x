SECTIONS
{
  .binfmt (INFO) :
  {
    /* Format implementations for primitives like u8 */
    *(.binfmt.prim.*);

    /* ERROR logging level */
    _binfmt_error = .;
    *(.binfmt.error.*);

    /* WARN logging level */
    _binfmt_warn = .;
    *(.binfmt.warn.*);

    /* INFO logging level */
    _binfmt_info = .;
    *(.binfmt.info.*);

    /* DEBUG logging level */
    _binfmt_debug = .;
    *(.binfmt.debug.*);

    /* TRACE logging level */
    _binfmt_trace = .;
    *(.binfmt.trace.*);

    /* Format/write! strings */
    _binfmt_fmt = .;
    *(.binfmt.fmt.*);

    /* User interned strings (Str) */
    _binfmt_str = .;
    *(.binfmt.str.*);
    _ebinfmt = .
  }
}
