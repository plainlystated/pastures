# pastures

Ranks your live AI coding sessions by *your* engagement, not the agent's.

Every existing session tool sorts by agent activity — file mtime, last status change, who pinged
most recently. That's the loudest and dumbest attention signal: a session that just pinged you
doesn't need help finding you. `pastures` ignores what the agent did and ranks by what you did:
how much thinking you put into a session, and how recently. The sessions you're toggling between
sit at the top; the ones that have gone warm — real work from earlier today that slipped out of
your head — sit right under them, easy to find. Cold ones sink.

**Status:** early but working. Claude Code only; GitHub-only install for now.

## Install

```sh
cargo install --git https://github.com/plainlystated/pastures pastures
```

## Use

```
$ pastures
SESSION                                  LAST  TURNS  LIVE   BRANCH   DIR
Jenkins build parallelization on GKE      14m     18  shell  master   ~/repos/cjp/workbench.git/main
Neglect-ranked session view for agents     5m     12  busy   master   ~/repos/personal/pastures.git/master
Airbrake review last 2 weeks               9m      9  idle   master   ~/repos/cjp/workbench.git/main
PR status                                 42m      5  idle   issue-…  ~/repos/cjp/workbench.git/ralph/issue-5455
Claude herd hook verification             12h     20  shell  -        ~/Sync/ai_workspaces/personal/personal-admin
...
```

`LAST` is time since *you* last typed something. `TURNS` is how many things you've typed. `LIVE`
is what the agent is doing (`busy`, `idle`, `shell`, or `?` when the process published no status).

- `pastures --json` — the same records as JSON, for adapters.
- `pastures --scores` — show the warmth column.
- `pastures --dump-config` — print the defaults; save to `~/.config/pastures/config.toml` to tune.

### herdr

A popup that lists the sessions under the current herdr server and focuses the one you pick:

```sh
herdr plugin install plainlystated/pastures/plugins/herdr
```

Then bind it (herdr plugins can't add keys themselves):

```toml
[[keys.command]]
key = "prefix+w"
type = "plugin_action"
command = "pastures.open"
```

Rows are labelled with the herdr tab name by default; set `PASTURES_HERDR_LABEL` to `workspace`,
`title`, or `pastures` to change that. Needs `pastures`, `jq`, and `fzf` on PATH.

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
  that churned autonomously for twenty minutes after you walked away has not made that session
  any warmer to you. Every tool surveyed uses file mtime and inherits the error.
- **Investment counts your turns, not wall-clock and not total messages.** Wall-clock lies — a
  session left open over lunch looks expensive and isn't — and total message count inflates under
  a long autonomous loop.

## Ranking

Warmth = `investment / staleness`. A session you put forty turns into an hour ago outranks one you
asked a single question ten minutes ago. Cheap sessions never crowd out real work, and real work
you've drifted away from stays findable instead of scrolling off.

Only sessions with a running agent process are listed. Closing a session removes it — that's the
dismiss action. There is no time window and nothing ages out.

The weighting is a couple of tunable exponents, not a plugin system.

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
report the review queue becoming *daunting*. That's why nothing here ages out on a timer and the
only dismiss is closing the session: the list is exactly what's open, never a backlog.

## Name

A pasture is where something goes when it's still alive and no longer in front of you. It also
contains *past*, which is what this asks you to grapple with.

An earlier draft of this README pitched the opposite ranking — neglect first, oldest and most
invested on top. Working through concrete scenarios showed that with closed sessions leaving the
list, "cold" mostly means "forgot to close it", and what actually needs finding is *warm*.

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
