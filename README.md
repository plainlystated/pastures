# pastures

Ranks your AI coding sessions by neglect, not recency.

Recency is the loudest and dumbest attention signal, and every existing session tool amplifies it.
A session that just pinged you doesn't need help finding you — it already found you. `pastures`
surfaces the opposite: work you sank real thinking into that has gone quiet and can't advocate
for itself.

**Status:** early. Design settled, implementation in progress. Nothing installable yet.

## The three metrics

| Metric | Property of | What it means |
| --- | --- | --- |
| **Liveness** | the agent | running / mid-turn / idle / dead |
| **Staleness** | you | how long since you last engaged |
| **Investment** | you | how much of your own thinking went in |

Liveness and staleness are orthogonal and frequently disagree. A dead session untouched for four
hours is exactly the thing that gets forgotten.

Two measurement choices follow from staleness and investment being properties of *you*, not of the
agent:

- **Staleness reads from the timestamp of your last turn, not the transcript's mtime.** An agent
  that churned autonomously for twenty minutes after you walked away has not reduced your neglect
  of that session. Every tool surveyed uses file mtime and inherits the error.
- **Investment counts your turns, not wall-clock and not total messages.** Wall-clock lies — a
  session left open over lunch looks expensive and isn't — and total message count inflates under
  a long autonomous loop.

## Ranking

Roughly `investment × staleness`. Cheap sessions decay quietly to the bottom; heavily-invested
neglected ones climb. Neither axis catches the target case alone: something you sank real work
into, that's paused or blocked, where you need to make a decision about it rather than merely
having forgotten it exists.

The weighting between the two factors is a couple of tunable numbers, not a plugin system.

**Liveness is not a sort key.** Sorting or grouping by it fragments the one list this exists to let
you scan. It's a small annotation on the row, relevant only once you've already decided to look.

## Architecture

**Core** — reads Claude Code and Codex transcripts from disk and emits neutral session records:
turn counts, first and last activity, working directory, liveness. Ranking lives here too; it's
arithmetic over data the core already holds. Useful on its own — every dashboard project in this
space reimplements this layer from scratch.

**Adapter** — given core records, supplies (a) a human label and (b) a way to focus a session. A
tmux adapter joins on pane titles; a [herdr](https://herdr.dev) adapter on its socket API. Small
by design; nothing gets pushed into the core for hypothetical adapters at real cost.

**CLI** — prints the ranked list to stdout. This is the primary product, not a demo. With no
adapter present it falls back to the working-directory basename, or the git branch, which for
worktree users is often the truest description of what a session is.

## Non-goals

- Not a timeline visualisation. This isn't chronological.
- No tool-use history, token analytics, or per-session event streams. That's a debugger; this is a
  triage list.
- No orchestration or agent coordination.

## Reading strategy

Parses the transcript JSONL directly. `sessions-index.json` is used at most as an opportunistic
cache for labels and summaries, always with JSONL fallback.

That's deliberate. The index has a sustained history of omitting sessions that exist on disk —
[#22205](https://github.com/anthropics/claude-code/issues/22205),
[#24729](https://github.com/anthropics/claude-code/issues/24729),
[#25032](https://github.com/anthropics/claude-code/issues/25032),
[#48334](https://github.com/anthropics/claude-code/issues/48334),
[#18897](https://github.com/anthropics/claude-code/issues/18897) — and the failures share a shape:
the JSONL survives, the index is what breaks. A tool whose entire job is surfacing what you forgot
cannot be built on a source whose documented failure mode is silently dropping rows.

## Prior art

Twenty-one tools read this same session data. All of them sort by recency, by liveness, or by a
board you arrange yourself. None has an investment axis.

[hex/claude-sessions](https://github.com/hex/claude-sessions) is the closest relative and worth
reading. It already separates staleness from liveness — a heat dot cooling green through gold and
orange to grey, a relative age column, a distinct marker for sessions with a live conversation.
But it's a session manager with its own workspace protocol rather than a passive reader of your
existing sessions, and its picker still opens most-recently-modified first.

[craftzdog/tmux-claude-session-manager](https://github.com/craftzdog/tmux-claude-session-manager)
is the only other tool that consciously refuses recency sorting — and replaces it with liveness,
promoting exactly the sessions already capable of interrupting you.

Outside this ecosystem, the nearest ancestor is OmniFocus's Review perspective: projects surfaced
on a cadence so they don't fall through the cracks. It's schedule-based rather than neglect-scored
and has no investment weighting, but it contributes a warning worth designing against — users
report the review queue becoming *daunting*. A neglect list with no way to acknowledge and dismiss
becomes a guilt pile you stop opening.

## Name

A pasture is where something goes when it's still alive and no longer in front of you. It also
contains *past*, which is what this asks you to grapple with.

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
