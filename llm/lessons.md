# futou Lessons

This is the canonical lessons file for the project. Existing lessons are kept in
[`lesson.md`](lesson.md) until that file is removed in a dedicated documentation
cleanup.

## Configuration must reach its consumer

Persisting a GUI setting does not change daemon behavior unless the daemon reads
or receives it.

**Rule:** Trace every setting from UI input to the process or service that owns
the behavior, and test the final resolved value.

## Startup readiness is asynchronous

Starting a child process does not mean its IPC endpoint is ready.

**Rule:** After starting the daemon, wait for a successful status request before
loading daemon-backed state. A successful readiness transition must trigger the
initial reload automatically.

## Historical lessons

See [`lesson.md`](lesson.md) for lessons captured before this canonical filename
was introduced.
