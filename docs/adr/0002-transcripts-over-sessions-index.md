# Read transcript JSONL directly; never depend on sessions-index.json

Claude Code writes a per-project `sessions-index.json` that would make listing sessions trivial.
We parse the transcript JSONL files instead and treat the index as untrusted. The index has a
documented history of omitting sessions that exist on disk (anthropics/claude-code #22205, #24729,
#25032, #48334, #18897), and on the reference machine it existed in 1 of 127 project directories,
pointing at a transcript that no longer existed. A tool whose job is showing you what you've lost
track of cannot be built on a source whose failure mode is silently dropping rows. Do not
"optimise" this back to the index.
