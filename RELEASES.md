Version 0.0.3  (2026-11-13)
===============================

   * The `minbas78` *compiler* now works with latest `rustc 1.100.0 (0ef1bc386 2026-11-09)` stable version and the following configurations:
      * Linux:   `x86_64-unknown-linux-musl` target.
      * Windows: `x86_64-pc-windows-msvc` target with the `synchronization.lib` library support. 
   
   * All of the `mindmap` *grammar rules*, including:
      * *Chapter 4. Characters and Strings*
      * *Chapter 5. Programs*
      * *Chapter 6. Constants*
      * *Chapter 7. Variables*
      * *Chapter 8. Expressions*
      * *Chapter 9. Implementation Supplied Functions*
      * *Chapter 10. User Defined Functions*
      * *Chapter 11. `LET` Statement*
      * *Chapter 12. Control Statement*
      * *Chapter 13. `FOR` and `NEXT` Statements*
      * *Chapter 14. `PRINT` statement*
      * *Chapter 15. `INPUT` statement*
      * *Chapter 16. `READ` and `RESTORE` statements*
      * *Chapter 17. `DATA` statement*
      * *Chapter 18. Array Declarations*
      * *Chapter 19. Remark Statement*
      * *Chapter 20. Randomize Statement*
      * *Table 1*


Version 0.0.2  (2026-11-12)
===============================

   * The `minbas78` *compiler* now works with the following configurations:
      * Linux:   `x86_64-unknown-linux-musl` target.
      * Windows: `x86_64-pc-windows-msvc` target.


Version 0.0.1  (2026-10-20)
===============================

   * `minbas78`, the *Minimal BASIC 78* language features work, including:
      * the *Abstract Syntax Tree*,
      * all the *external built-in "C" functions* and the support for *user defined functions*,
      * the *grammar rules*,
      * the *walker* and *visitor* patterns implementations,
      * utilities constants, functions and helper methods,
      * the *system*, *compiler* and *runtime* errors.

   * The *compiler* works with the following configurations:
      * Windows: `x86_64-pc-windows-msvc` target.

   * Includes the **Dartmouth Manual** Examples.
   
   * The *Markdown documentation* and `doctests`.
