<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { t } from '../i18n';

  const nativeWindow = getCurrentWindow();
  const edges = [
    'North',
    'NorthEast',
    'East',
    'SouthEast',
    'South',
    'SouthWest',
    'West',
    'NorthWest',
  ] as const;
  let maximized = false;
  let fullscreen = false;
  let failed = false;

  async function perform(action: () => Promise<unknown>) {
    failed = false;
    try {
      await action();
    } catch {
      failed = true;
    }
  }

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    async function syncState() {
      const [isMaximized, isFullscreen] = await Promise.all([
        nativeWindow.isMaximized(),
        nativeWindow.isFullscreen(),
      ]);
      if (!disposed) {
        maximized = isMaximized;
        fullscreen = isFullscreen;
      }
    }
    void perform(async () => {
      const stop = await nativeWindow.onResized(() => {
        void perform(syncState);
      });
      if (disposed) {
        stop();
        return;
      }
      unlisten = stop;
      await syncState();
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

{#if !fullscreen}
  <!-- Window controls below provide the keyboard equivalent of double-clicking this area. -->
  <button
    class="linux-window-drag"
    tabindex="-1"
    aria-label={maximized ? $t.window.restore : $t.window.maximize}
    onmousedown={(event) => {
      if (event.button !== 0) return;
      void perform(() =>
        event.detail === 2 ? nativeWindow.toggleMaximize() : nativeWindow.startDragging(),
      );
    }}
    onclick={(event) => {
      if (event.detail === 0) void perform(() => nativeWindow.toggleMaximize());
    }}
  ></button>
  <div class="linux-window-controls">
    <button
      aria-label={$t.window.minimize}
      title={$t.window.minimize}
      onclick={() => void perform(() => nativeWindow.minimize())}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true"><path d="M3 11h10" /></svg>
    </button>
    <button
      aria-label={maximized ? $t.window.restore : $t.window.maximize}
      title={maximized ? $t.window.restore : $t.window.maximize}
      onclick={() => void perform(() => nativeWindow.toggleMaximize())}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">
        {#if maximized}<path d="M5 5V3h8v8h-2M3 5h8v8H3z" />{:else}<path d="M3 3h10v10H3z" />{/if}
      </svg>
    </button>
    <button
      class="window-close"
      aria-label={$t.window.close}
      title={$t.window.close}
      onclick={() => void perform(() => nativeWindow.close())}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true"
        ><path d="m3 3 10 10M13 3 3 13" /></svg
      >
    </button>
  </div>
  {#if !maximized}
    {#each edges as edge (edge)}
      <div
        class={`window-resize-edge edge-${edge}`}
        aria-hidden="true"
        onpointerdown={(event) => {
          if (event.button === 0) void perform(() => nativeWindow.startResizeDragging(edge));
        }}
      ></div>
    {/each}
  {/if}
{/if}
{#if failed}<p class="sr-only" role="status">{$t.window.actionFailed}</p>{/if}
