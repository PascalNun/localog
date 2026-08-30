import { strings } from '../i18n';

/**
 * What to call a protocol style.
 *
 * A style's name is stored in the database, seeded there by the migration that
 * creates the three LocaLog ships with. That makes it data — somebody can rename
 * a style, and what they rename it to is theirs — but it also meant a French
 * interface offered “Formal minutes”, “Internal working note” and “Technical
 * decision log” as the whole of its choice about how a document reads.
 *
 * So the words below are used under one condition: the style is one of the three,
 * and it still carries the text it shipped with. Rename it and the name you gave
 * it is what you see, in every language, because at that point it is not the
 * shipped style any more.
 *
 * The English text is repeated here on purpose, and it is the only place in the
 * interface that repeats a stored value. It is a *comparison*, not a label: if
 * the migration ever changes the seeded text, this stops matching and the stored
 * name shows through — which is the harmless direction.
 */
type ShippedStyleId = 'style-formal' | 'style-working-note' | 'style-decision-log';

const AS_SHIPPED: Record<ShippedStyleId, { name: string; description: string }> = {
  'style-formal': {
    name: 'Formal minutes',
    description: 'Structured record of discussion, decisions, and actions.',
  },
  'style-working-note': {
    name: 'Internal working note',
    description: 'Concise working record for an internal project team.',
  },
  'style-decision-log': {
    name: 'Technical decision log',
    description: 'Emphasises alternatives, constraints, and explicit decisions.',
  },
};

function shippedId(id: string): ShippedStyleId | null {
  return id in AS_SHIPPED ? (id as ShippedStyleId) : null;
}

export function styleName(style: { id: string; name: string }): string {
  const id = shippedId(style.id);
  if (id === null || style.name !== AS_SHIPPED[id].name) return style.name;
  return strings().library.shippedStyle[id].name;
}

export function styleDescription(style: { id: string; description: string }): string {
  const id = shippedId(style.id);
  if (id === null || style.description !== AS_SHIPPED[id].description) return style.description;
  return strings().library.shippedStyle[id].description;
}
