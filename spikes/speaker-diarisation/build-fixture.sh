#!/bin/sh
# Generates a synthetic multi-speaker German meeting with exact ground truth.
# No real meeting content: every line is invented for testing.
set -e
OUT=fixtures
mkdir -p "$OUT/parts"
rm -f "$OUT/parts"/*.aiff "$OUT/parts"/*.wav 2>/dev/null || true

# speaker|voice|text
LINES='S1|Anna|Guten Morgen, wir beginnen mit dem Statusbericht zur Fassade.
S2|Eddy|Die Lieferung der Verglasung verschiebt sich um zwei Wochen.
S1|Anna|Das betrifft dann auch den Innenausbau im dritten Obergeschoss.
S3|Sandy|Ich stimme mich dazu mit dem Statiker ab und melde mich Donnerstag.
S2|Eddy|Wir sollten die Kosten für die Alternative ebenfalls prüfen.
S1|Anna|Gut, damit halten wir zwei offene Punkte fest und vertagen die Entscheidung.'

i=0
echo "$LINES" | while IFS='|' read -r spk voice text; do
  i=$((i+1))
  n=$(printf "%02d" $i)
  say -v "$voice" -o "$OUT/parts/$n-$spk.aiff" "$text"
done
