# Pastures

Ranks a person's live AI coding sessions by their own engagement — how much they put in and how recently — ignoring what the agent has been doing, so that work that has gone warm is easy to find among the hot sessions they already know about.

## Language

**Session**:
One conversation between the person and a coding agent, identified by the agent's session id and backed by a transcript on disk. Only sessions with a running agent process are ranked; closing the process removes the session.
_Avoid_: conversation, chat, tab, task

**Turn**:
One message the person typed to the agent. Tool results, system-injected messages, and agent replies are not turns.
_Avoid_: message, prompt, exchange

**Investment**:
How much of the person's own thinking has gone into a session. Measured in turns.
_Avoid_: effort, cost, size, weight, duration

**Staleness**:
How long since the person last engaged with a session, measured from their last turn. A property of the person, not of the agent.
_Avoid_: age, idle time, last activity, mtime

**Liveness**:
What the agent is doing right now: running, mid-turn, idle, or dead. A property of the agent. Never a sort key.
_Avoid_: status, state, activity, health

**Warmth**:
The ranking score: investment divided by staleness. Hot sessions (touched just now) rank first, warm ones next, cold ones last.
_Avoid_: neglect, priority, urgency, score, rank, heat

**Hot / Warm / Cold**:
Informal bands of warmth. Hot is what the person is toggling between right now; warm is what they were in earlier today and may have lost track of; cold is open but abandoned.
_Avoid_: active, stale, dead (dead is a liveness value, not a warmth band)

**Transcript**:
The JSONL file the agent itself writes for a session. Read as-is; never summarised or processed by a model.
_Avoid_: log, history, chat log

**Label**:
The human-readable name shown for a session. Supplied by an adapter when one is present, otherwise derived from the session itself.
_Avoid_: title, name, summary

**Adapter**:
A small integration that, given sessions, supplies labels and a way to focus a session in a particular host (herdr, tmux).
_Avoid_: plugin, backend, driver, integration
