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
#   pane_id \t space \t label \t last \t turns \t live \t branch
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
  | ($ws_label[$a.workspace_id] | named // "-") as $space
  | [$a.pane_id, $space, $label, rel($r.staleness_hours), ($r.turns|tostring), $a.agent_status, $branch]
  | @tsv
')

if [[ -z "$lines" ]]; then
  echo "no live sessions in this herdr" >&2
  sleep 1.5
  exit 0
fi

# Header goes through the same alignment as the rows; fzf hides the pane-id column from both.
# Colour is applied after alignment (escape codes would throw off column widths):
# working agents are dimmed, blocked ones (waiting on a permission or a question) are bold.
header=$'_\tSPACE\tSESSION\tLAST\tTURNS\tLIVE\tBRANCH'
dim=$'\e[2m'; bold=$'\e[1m'; reset=$'\e[0m'
choice=$(printf '%s\n%s\n' "$header" "$lines" | column -t -s $'\t' -o '  ' \
  | awk -v d="$dim" -v b="$bold" -v r="$reset" '
      NR == 1          { print; next }
      /  working  /    { print d $0 r; next }
      /  blocked  /    { print b $0 r; next }
                       { print }' \
  | fzf --ansi --no-sort --with-nth=2.. --header-lines=1 --prompt='warm> ' --layout=reverse) || exit 0

pane=$(awk '{print $1}' <<<"$choice")
exec "$herdr" agent focus "$pane"
