/**
 * What changed between two versions of a passage.
 *
 * A rewrite from a local model cannot be trusted to keep the facts — measured, not
 * feared: across nine rewrites of three German passages it altered a fact in three
 * of twenty-four. Checking the result afterwards catches some of that and not all
 * of it, because "KW 38" becoming "Woche 38" loses nothing a checker can count.
 *
 * So the answer is not a better checker. It is to show the change and wait: if
 * nothing lands unread, the model being unreliable stops mattering. This is the
 * part that shows it.
 *
 * Word-level rather than character-level, because a person reading a change wants
 * to see which words moved, not which letters did.
 */

export type Change =
  | { kind: 'same'; text: string }
  | { kind: 'removed'; text: string }
  | { kind: 'added'; text: string };

/** Beyond this, the table below would cost more than the answer is worth. */
const MOST_WORDS = 1_200;

/**
 * Split into words while keeping the spaces, so that joining the pieces back
 * together returns exactly what came in.
 */
function intoWords(text: string): string[] {
  return text.match(/\s+|[^\s]+/g) ?? [];
}

export function diffWords(before: string, after: string): Change[] {
  const left = intoWords(before);
  const right = intoWords(after);

  if (left.length > MOST_WORDS || right.length > MOST_WORDS) {
    // Too long to line up word by word; say so as one replacement rather than
    // pretending to a detail this cannot supply.
    return [
      { kind: 'removed', text: before },
      { kind: 'added', text: after },
    ];
  }

  // The classic longest-common-subsequence table. Rows are the left side, columns
  // the right; each cell holds the length of the best match of the two suffixes.
  const table: number[][] = Array.from({ length: left.length + 1 }, () =>
    new Array<number>(right.length + 1).fill(0),
  );
  for (let l = left.length - 1; l >= 0; l -= 1) {
    for (let r = right.length - 1; r >= 0; r -= 1) {
      const row = table[l];
      const below = table[l + 1];
      if (!row || !below) continue;
      row[r] =
        left[l] === right[r] ? (below[r + 1] ?? 0) + 1 : Math.max(below[r] ?? 0, row[r + 1] ?? 0);
    }
  }

  const changes: Change[] = [];
  const push = (kind: Change['kind'], text: string) => {
    const last = changes[changes.length - 1];
    if (last && last.kind === kind) last.text += text;
    else changes.push({ kind, text } as Change);
  };

  let l = 0;
  let r = 0;
  while (l < left.length && r < right.length) {
    if (left[l] === right[r]) {
      push('same', left[l] ?? '');
      l += 1;
      r += 1;
      continue;
    }
    const down = table[l + 1]?.[r] ?? 0;
    const across = table[l]?.[r + 1] ?? 0;
    if (down >= across) {
      push('removed', left[l] ?? '');
      l += 1;
    } else {
      push('added', right[r] ?? '');
      r += 1;
    }
  }
  while (l < left.length) {
    push('removed', left[l] ?? '');
    l += 1;
  }
  while (r < right.length) {
    push('added', right[r] ?? '');
    r += 1;
  }

  return changes;
}

/** Whether anything actually changed, so an identical rewrite can say so. */
export function isUnchanged(changes: Change[]): boolean {
  return changes.every((change) => change.kind === 'same' || change.text.trim() === '');
}
