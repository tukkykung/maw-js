#!/bin/bash
# maw overview mirror — bottom-aligns captured pane content
TARGET="$1"
SEP="────────────────────────────────────────────────────────────"
h=$(tput lines)
o=$(tmux capture-pane -t "$TARGET" -e -p 2>/dev/null | sed -E "s/[─━]{6,}/$SEP/g" | grep -v '^$' | tail -$h)
n=$(echo "$o" | wc -l)
pad=$((h - n))
[ $pad -gt 0 ] && printf '\n%.0s' $(seq 1 $pad)
echo "$o"
