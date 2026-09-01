# The list is exactly the sessions with a live agent process

Membership comes from `~/.claude/sessions/<pid>.json`, which Claude Code (2.1.2xx) maintains for
each running process with `sessionId`, `cwd`, `status` (`idle|busy|shell`), and a `procStart`
guard against pid reuse; the file is removed on exit. This couples pastures to an undocumented file,
and processes started by older versions may only have a `.key` file (shown as liveness `?`, never
hidden). We accepted that over the alternatives: scanning all transcripts with a time window
(needs aging-out rules and a dismiss mechanism, and every transcript's mtime is unreliable), or
process-table sniffing (no reliable pid→session mapping; cmdline only carries the id for explicit
`--resume`). Closing a session is the only dismiss; nothing ages out on a timer.
