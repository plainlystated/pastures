#!/usr/bin/env bash
# pastures — herdr popup adapter.
#
# Joins `pastures --json` to this herdr session's agent panes on the Claude session id, shows them
# warmest first in fzf, and focuses the chosen pane. Only sessions running under *this* herdr
# server appear, so a `personal` server never lists `cjp` work.
#
# Label source (PASTURES_HERDR_LABEL): tab (default) | workspace | title | pastures
#   tab        the herdr tab name
#   workspace  the herdr workspace name
#   title      the agent's terminal title
#   pastures   whatever pastures itself derived (/rename title, ai title, first message, dir)
set -euo pipefail

herdr="${HERDR_BIN_PATH:-herdr}"
label_mode="${PASTURES_HERDR_LABEL:-tab}"

for dep in pastures jq fzf; do
  command -v "$dep" >/dev/null || { echo "pastures popup: $dep is not on PATH" >&2; sleep 2; exit 1; }
done

records=$(pastures --json)
agents=$("$herdr" agent list)
tabs=$("$herdr" tab list)
workspaces=$("$herdr" workspace list)

# One line per joined session, pastures' order preserved:
#   pane_id \t label \t last \t turns \t live \t branch
lines=$(jq -rn \
  --argjson records "$records" \
  --argjson agents "$agents" \
  --argjson tabs "$tabs" \
  --argjson workspaces "$workspaces" \
  --arg mode "$label_mode" '
  def rel($h):
    if $h == null then "-"
    elif $h * 60 < 1 then "now"
    elif $h < 1 then "\(($h * 60) | floor)m"
    elif $h < 24 then "\($h | floor)h"
    else "\(($h / 24) | floor)d" end;
  # herdr names unnamed tabs/workspaces by number; treat those as no name.
  def named: if . == null or test("^[0-9]+$") then null else . end;

  ($agents.result.agents // []) as $ag
  | ($tabs.result.tabs // [] | map({(.tab_id): .label}) | add // {}) as $tab_label
  | ($workspaces.result.workspaces // [] | map({(.workspace_id): .label}) | add // {}) as $ws_label
  | $records[]
  | select(.session_id != null)
  | . as $r
  | ($ag[] | select(.agent_session? and .agent_session.value == $r.session_id)) as $a
  | (if $mode == "workspace" then ($ws_label[$a.workspace_id] | named)
     elif $mode == "title" then $a.terminal_title_stripped
     elif $mode == "pastures" then $r.label
     else ($tab_label[$a.tab_id] | named) end // $r.label) as $label
  | ($r.git_branch | if . == null or . == "HEAD" then "-" else . end) as $branch
  | [$a.pane_id, $label, rel($r.staleness_hours), ($r.turns|tostring), $a.agent_status, $branch]
  | @tsv
')

if [[ -z "$lines" ]]; then
  echo "no live sessions in this herdr" >&2
  sleep 1.5
  exit 0
fi

choice=$(printf '%s\n' "$lines" | column -t -s $'\t' -o '  ' \
  | fzf --no-sort --with-nth=2.. --prompt='warm> ' \
        --header="$(printf 'SESSION  LAST  TURNS  LIVE  BRANCH')" \
        --layout=reverse) || exit 0

pane=$(awk '{print $1}' <<<"$choice")
exec "$herdr" agent focus "$pane"
