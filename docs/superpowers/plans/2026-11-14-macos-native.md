# macOS native clipboard and original output

**Goal:** Support only macOS, directly use NSPasteboard, and preserve the original input on stdout.

**Requirements:** Default clipboard payload is `input.trim()` (Unicode whitespace). `-f`/`--full` copies the exact original input. Default stdout preserves the original bytes, adds a separating newline only when nonempty input lacks a trailing LF, then prints `zcco copied.` on its own line. Empty input prints only `zcco copied.`. `-q`/`--quiet` suppresses both original text and the confirmation; errors remain visible. All existing UTF-8 validation and error exit behavior remains.

**Approach:** Use objc2, objc2-app-kit (NSPasteboard only) and objc2-foundation (NSString only). No external command or background clipboard owner. Remove Linux backends and musl linking configuration; retain Intel and Apple Silicon macOS CI builds. User requested implementation in the current checkout; no commits or global executable installation.

- [x] Write tests for original output, trim/full payload selection, quiet combinations, blank input and failure behavior; confirm missing new interfaces fail before implementation.
- [x] Add a native test using a unique named pasteboard, without touching the user's general pasteboard.
- [x] Implement native clipboard writing and CLI argument handling, then run all tests.
- [x] Update README, help, platform metadata and CI; remove obsolete Linux linking configuration.
- [x] Run final formatting, Clippy, tests and both macOS release builds.
- [x] Obtain independent read-only review and address any material findings.

**Validation note:** Sandbox access to the pasteboard service returned nil. Running the tests with authorized desktop access passed. Production code treats nil as an actionable error instead of panicking.


**Final verification:** Formatting and Clippy passed; all 11 tests passed with authorized desktop service access (8 unit/native, 3 CLI). Native release and both aarch64-apple-darwin/x86_64-apple-darwin release builds passed; `file` confirmed the two Mach-O architectures. Independent read-only review approved the revised scope with no Critical/Important findings. The user's general clipboard was not modified; Intel runtime and hosted CI execution have not been exercised. Success confirmation is `zcco copied.` per the final user request.
