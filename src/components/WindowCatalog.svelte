<script lang="ts">
  import { windows } from "$lib/stores/windows";
  import { assignedWindowIds, assignments } from "$lib/stores/assignments";
  import SearchBox from "./SearchBox.svelte";
  import WindowCard from "./WindowCard.svelte";

  interface Props {
    onDragStart: (windowId: string, x: number, y: number) => void;
  }

  let { onDragStart }: Props = $props();

  let search = $state("");

  const filteredWindows = $derived(
    $windows.filter((w) => {
      if (!search) return true;
      const q = search.toLowerCase();
      return w.app_name.toLowerCase().includes(q) || w.title.toLowerCase().includes(q);
    })
  );

  function getAssignedZone(windowId: string): number | null {
    for (const [key, val] of Object.entries($assignments)) {
      if (val === windowId) return Number(key);
    }
    return null;
  }
</script>

<div class="catalog-panel">
  <div class="panel-header">
    <div class="panel-title">Window Catalog</div>
    <SearchBox bind:value={search} />
  </div>
  <ul class="window-list">
    {#each filteredWindows as win (win.id)}
      <WindowCard
        window={win}
        isAssigned={$assignedWindowIds.has(win.id)}
        assignedZone={getAssignedZone(win.id)}
        {onDragStart}
      />
    {/each}
    {#if filteredWindows.length === 0}
      <div class="empty-list">No windows found</div>
    {/if}
  </ul>
</div>

<style>
  .catalog-panel {
    background: var(--surface); border-right: 1px solid var(--border);
    display: flex; flex-direction: column; overflow: hidden;
  }
  .panel-header { padding: 16px 16px 8px; }
  .panel-title {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.08em; color: var(--text-mute); margin-bottom: 12px;
  }
  .window-list { flex: 1; overflow-y: auto; padding: 8px; list-style: none; }
  .empty-list { padding: 24px 16px; text-align: center; color: var(--text-mute); font-size: 13px; }
</style>
