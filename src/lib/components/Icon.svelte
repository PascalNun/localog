<script context="module" lang="ts">
  export type IconName =
    | 'arrow'
    | 'book'
    | 'check'
    | 'chevron'
    | 'close'
    | 'document'
    | 'download'
    | 'folder'
    | 'info'
    | 'menu'
    | 'moon'
    | 'pause'
    | 'play'
    | 'plus'
    | 'search'
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
  {:else if name === 'moon'}
    <path d="M19 15.5A8 8 0 0 1 8.5 5 8 8 0 1 0 19 15.5" />
  {:else if name === 'pause'}
    <path d="M9 7v10M15 7v10" />
  {:else if name === 'play'}
    <path d="m9 7 8 5-8 5z" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
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
