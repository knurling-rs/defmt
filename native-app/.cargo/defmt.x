EXTERN(_defmt_acquire);
EXTERN(_defmt_release);
EXTERN(__defmt_default_timestamp);
EXTERN(__DEFMT_MARKER_TIMESTAMP_WAS_DEFINED);
PROVIDE(_defmt_timestamp = __defmt_default_timestamp);
PROVIDE(_defmt_panic = __defmt_default_panic);

SECTIONS
{
  .defmt :
  {
    /* Format implementations for primitives like u8 */
    KEEP(*(.defmt.prim.*));

    /* We order the ids of the log messages by severity and put markers in between, so that we can filter logs at runtime by severity */
    __DEFMT_MARKER_TRACE_START = .;
    KEEP(*(.defmt.trace.*));
    __DEFMT_MARKER_TRACE_END = .;
    __DEFMT_MARKER_DEBUG_START = .;
    KEEP(*(.defmt.debug.*));
    __DEFMT_MARKER_DEBUG_END = .;
    __DEFMT_MARKER_INFO_START = .;
    KEEP(*(.defmt.info.*));
    __DEFMT_MARKER_INFO_END = .;
    __DEFMT_MARKER_WARN_START = .;
    KEEP(*(.defmt.warn.*));
    __DEFMT_MARKER_WARN_END = .;
    __DEFMT_MARKER_ERROR_START = .;
    KEEP(*(.defmt.error.*));
    __DEFMT_MARKER_ERROR_END = .;

    /* Everything user-defined */
    KEEP(*(.defmt.*));

    __DEFMT_MARKER_END = .;

    /* Symbols that aren't referenced by the program and */
    /* should be placed at the end of the section */
    KEEP(*(.defmt.end .defmt.end.*));
  }
} INSERT AFTER .rodata;
