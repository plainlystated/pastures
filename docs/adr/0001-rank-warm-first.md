# Rank warm-first, not neglect-first

The original thesis (and the first README) ranked sessions by `investment × staleness` so the
oldest, most-invested session sat on top. Ranking eight concrete live sessions showed that reading
was wrong for daily use: once closed sessions leave the list, a cold session almost always means
"forgot to close it", and what actually needs finding is work that has gone *warm* while you toggle
between the two or three hot ones you already know. Warmth is therefore `investment / staleness`;
hot on top, warm beneath, cold at the bottom where a short list still makes it visible. The
alternative — a hump that peaks at warm and decays both ways — was rejected as one tunable more
than the problem needs.
