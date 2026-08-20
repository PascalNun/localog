<script context="module" lang="ts">
  export type IconName =
    | 'arrow'
    | 'bold'
    | 'italic'
    | 'link'
    | 'list-bulleted'
    | 'list-numbered'
    | 'quote'
    | 'rewrite'
    | 'rule'
    | 'table'
    | 'column-add-left'
    | 'column-add-right'
    | 'column-remove'
    | 'row-add-above'
    | 'row-add-below'
    | 'row-remove'
    | 'book'
    | 'check'
    | 'chevron'
    | 'chevron-down'
    | 'close'
    | 'document'
    | 'download'
    | 'folder'
    | 'info'
    | 'menu'
    | 'monitor'
    | 'moon'
    | 'pause'
    | 'play'
    | 'plus'
    | 'redo'
    | 'search'
    | 'undo'
    | 'settings'
    | 'sun'
    | 'upload'
    | 'warning'
    | 'microphone';
</script>

<script lang="ts">
  export let name: IconName;
  export let size = 18;
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.7"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'arrow'}
    <path d="M5 12h14M14 7l5 5-5 5" />
  {:else if name === 'bold'}
    <path d="M7 5h5.5a3.5 3.5 0 0 1 0 7H7zM7 12h6.5a3.5 3.5 0 0 1 0 7H7z" />
  {:else if name === 'italic'}
    <path d="M15 5h-5M14 19H9M14 5l-4 14" />
  {:else if name === 'link'}
    <path d="M10.5 13.5a3.5 3.5 0 0 0 5 0l3-3a3.54 3.54 0 0 0-5-5l-1.5 1.5" />
    <path d="M13.5 10.5a3.5 3.5 0 0 0-5 0l-3 3a3.54 3.54 0 0 0 5 5L12 17" />
  {:else if name === 'list-bulleted'}
    <path d="M9 6h11M9 12h11M9 18h11" />
    <circle cx="4.5" cy="6" r="1.1" fill="currentColor" stroke="none" />
    <circle cx="4.5" cy="12" r="1.1" fill="currentColor" stroke="none" />
    <circle cx="4.5" cy="18" r="1.1" fill="currentColor" stroke="none" />
  {:else if name === 'list-numbered'}
    <path d="M10 6h10M10 12h10M10 18h10" />
    <path d="M4 5.5 5.3 5v3.4M3.4 15.2a1.4 1.4 0 1 1 2.4 1L3.4 19h2.6" stroke-width="1.5" />
  {:else if name === 'quote'}
    <path
      d="M9.5 7C7 8 5.5 10 5.5 12.6c0 2 1.2 3.4 3 3.4 1.6 0 2.8-1.2 2.8-2.8 0-1.5-1-2.6-2.4-2.6-.3 0-.6 0-.8.1.3-1 1.1-2 2.4-2.7z"
    />
    <path
      d="M18.5 7C16 8 14.5 10 14.5 12.6c0 2 1.2 3.4 3 3.4 1.6 0 2.8-1.2 2.8-2.8 0-1.5-1-2.6-2.4-2.6-.3 0-.6 0-.8.1.3-1 1.1-2 2.4-2.7z"
    />
  {:else if name === 'rewrite'}
    <path d="M12 3.5 13.4 8l4.5 1.4-4.5 1.4L12 15.3l-1.4-4.5L6.1 9.4 10.6 8z" />
    <path d="M18 15.5l.7 2.1 2.1.7-2.1.7-.7 2.1-.7-2.1-2.1-.7 2.1-.7z" />
  {:else if name === 'rule'}
    <!-- Text, a rule across the full measure, then text. Three equal lines read as
         a menu, which is what the first attempt looked like. -->
    <path d="M6 6.5h12M6 9.5h8" opacity="0.4" />
    <path d="M3 13.5h18" />
    <path d="M6 17.5h12M6 20.5h8" opacity="0.4" />
  {:else if name === 'table'}
    <rect x="3.5" y="4.5" width="17" height="15" rx="1.5" />
    <path d="M3.5 9.5h17M9.5 9.5v10M15 9.5v10" />
  {:else if name === 'row-add-above'}
    <rect x="3.5" y="12.5" width="17" height="7" rx="1.5" />
    <path d="M12 3.5v6M9 6.5h6" stroke-width="2" />
  {:else if name === 'row-add-below'}
    <rect x="3.5" y="4.5" width="17" height="7" rx="1.5" />
    <path d="M12 14.5v6M9 17.5h6" stroke-width="2" />
  {:else if name === 'row-remove'}
    <!-- The bar is the row; the stroke through it is what happens to it. A minus
         set outside the shape was too small to see at 15px. -->
    <rect x="3.5" y="7" width="17" height="7" rx="1.5" />
    <path d="M7.5 20h9" stroke-width="2.4" />
  {:else if name === 'column-add-left'}
    <rect x="13" y="3.5" width="7" height="17" rx="1.5" />
    <path d="M6.5 9v6M3.5 12h6" stroke-width="2" />
  {:else if name === 'column-add-right'}
    <rect x="4" y="3.5" width="7" height="17" rx="1.5" />
    <path d="M17.5 9v6M14.5 12h6" stroke-width="2" />
  {:else if name === 'column-remove'}
    <rect x="4" y="3.5" width="7" height="17" rx="1.5" />
    <path d="M15 12h6" stroke-width="2.4" />
  {:else if name === 'microphone'}
    <path d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3z" />
    <path d="M5 11v1a7 7 0 0 0 14 0v-1M12 19v2" />
  {:else if name === 'book'}
    <path
      d="M4 5.5A2.5 2.5 0 0 1 6.5 3H11v16H6.5A2.5 2.5 0 0 0 4 21.5zM20 5.5A2.5 2.5 0 0 0 17.5 3H13v16h4.5a2.5 2.5 0 0 1 2.5 2.5z"
    />
  {:else if name === 'check'}
    <path d="m5 12 4 4L19 6" />
  {:else if name === 'chevron'}
    <path d="m9 18 6-6-6-6" />
  {:else if name === 'chevron-down'}
    <path d="m6 9 6 6 6-6" />
  {:else if name === 'close'}
    <path d="m6 6 12 12M18 6 6 18" />
  {:else if name === 'document'}
    <path d="M6 3h8l4 4v14H6zM14 3v5h5M9 13h6M9 17h6" />
  {:else if name === 'download'}
    <path d="M12 3v12m-4-4 4 4 4-4M5 20h14" />
  {:else if name === 'folder'}
    <path d="M3 6h7l2 2h9v11H3z" />
  {:else if name === 'info'}
    <circle cx="12" cy="12" r="9" /><path d="M12 10v6M12 7h.01" />
  {:else if name === 'menu'}
    <path d="M4 7h16M4 12h16M4 17h16" />
  {:else if name === 'monitor'}
    <rect x="2" y="4" width="20" height="13" rx="2" />
    <path d="M8 21h8M12 17v4" />
  {:else if name === 'moon'}
    <path d="M19 15.5A8 8 0 0 1 8.5 5 8 8 0 1 0 19 15.5" />
  {:else if name === 'pause'}
    <path d="M9 7v10M15 7v10" />
  {:else if name === 'play'}
    <path d="m9 7 8 5-8 5z" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'undo'}
    <path d="M3 8h11a5 5 0 0 1 0 10h-6M3 8l4-4M3 8l4 4" />
  {:else if name === 'redo'}
    <path d="M21 8H10a5 5 0 0 0 0 10h6M21 8l-4-4M21 8l-4 4" />
  {:else if name === 'search'}
    <circle cx="11" cy="11" r="7" /><path d="m20 20-4-4" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" /><path
      d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.6v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1z"
    />
  {:else if name === 'sun'}
    <circle cx="12" cy="12" r="4" /><path
      d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
    />
  {:else if name === 'upload'}
    <path d="M12 16V4m-4 4 4-4 4 4M5 14v6h14v-6" />
  {:else if name === 'warning'}
    <path d="M12 3 2.5 20h19zM12 9v4M12 17h.01" />
  {/if}
</svg>
