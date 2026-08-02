<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    DEFAULT_SIDEBAR_WIDTH,
    MAX_SIDEBAR_WIDTH,
    MIN_SIDEBAR_WIDTH,
    clampSidebarWidth,
    sidebarWidthForKey,
  } from '../layout/sidebarSizing';

  export let width: number;
  export let onResize: (width: number) => void;
  export let onResizeEnd: (width: number) => void;

  let activePointerId: number | null = null;
  let startX = 0;
  let startWidth = width;
  let pendingWidth = width;

  function beginResize(event: PointerEvent) {
    if (event.button !== 0) return;
    event.preventDefault();
    activePointerId = event.pointerId;
    startX = event.clientX;
    startWidth = width;
    pendingWidth = width;
    const handle = event.currentTarget as HTMLElement;
    // Pointer capture keeps the drag coherent after the cursor leaves the narrow divider.
    handle.setPointerCapture(event.pointerId);
    document.body.classList.add('sidebar-resizing');
  }

  function continueResize(event: PointerEvent) {
    if (activePointerId !== event.pointerId) return;
    pendingWidth = clampSidebarWidth(startWidth + event.clientX - startX);
    onResize(pendingWidth);
  }

  function finishResize(event: PointerEvent) {
    if (activePointerId !== event.pointerId) return;
    const handle = event.currentTarget as HTMLElement;
    activePointerId = null;
    if (handle.hasPointerCapture(event.pointerId)) handle.releasePointerCapture(event.pointerId);
    document.body.classList.remove('sidebar-resizing');
    onResizeEnd(pendingWidth);
  }

  function handleLostPointerCapture(event: PointerEvent) {
    if (activePointerId !== event.pointerId) return;
    activePointerId = null;
    document.body.classList.remove('sidebar-resizing');
    onResizeEnd(pendingWidth);
  }

  function handleKeydown(event: KeyboardEvent) {
    const nextWidth = sidebarWidthForKey(width, event.key, event.shiftKey);
    if (nextWidth === null) return;
    event.preventDefault();
    onResize(nextWidth);
    onResizeEnd(nextWidth);
  }

  function resetWidth() {
    onResize(DEFAULT_SIDEBAR_WIDTH);
    onResizeEnd(DEFAULT_SIDEBAR_WIDTH);
  }

  onDestroy(() => document.body.classList.remove('sidebar-resizing'));
</script>

<!-- A focusable ARIA separator is the standard control for resizing adjacent panes. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex a11y_no_noninteractive_element_interactions -->
<div
  class="sidebar-resize-handle"
  role="separator"
  aria-label="Resize sidebar. Use arrow keys to adjust or Enter to reset."
  aria-orientation="vertical"
  aria-valuemin={MIN_SIDEBAR_WIDTH}
  aria-valuemax={MAX_SIDEBAR_WIDTH}
  aria-valuenow={width}
  aria-valuetext={`${width} pixels`}
  tabindex="0"
  onpointerdown={beginResize}
  onpointermove={continueResize}
  onpointerup={finishResize}
  onpointercancel={finishResize}
  onlostpointercapture={handleLostPointerCapture}
  onkeydown={handleKeydown}
  ondblclick={resetWidth}
></div>
