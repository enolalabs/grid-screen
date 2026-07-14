<script lang="ts">
  import { screens, selectedScreenId } from "$lib/stores/screen";

  let open = $state(false);

  function toggleOpen() {
    open = !open;
  }

  function selectScreen(id: string) {
    selectedScreenId.set(id);
    open = false;
  }

  function handleClickOutside(e: MouseEvent) {
    if (open) open = false;
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="selector" onclick={toggleOpen} role="button" tabindex="-1">
  <svg class="selector-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <rect x="2" y="3" width="20" height="14" rx="2"/>
    <line x1="8" y1="21" x2="16" y2="21"/>
    <line x1="12" y1="17" x2="12" y2="21"/>
  </svg>
  <div>
    <div class="selector-label">Screen</div>
    <div class="selector-value">
      {$screens.find((s) => s.id === $selectedScreenId)?.label ?? "None"}
    </div>
  </div>
  <div class="dropdown" class:open>
    {#each $screens as s (s.id)}
      <div
        class="dropdown-item"
        class:selected={s.id === $selectedScreenId}
        onclick={() => selectScreen(s.id)}
        role="option"
      >
        {s.label} &mdash; {s.resolution}
      </div>
    {/each}
  </div>
</div>

<style>
  .selector {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 14px; background: var(--surface);
    border: 1px solid var(--border); border-radius: var(--radius-sm);
    font-size: 13px; font-weight: 500; cursor: pointer;
    transition: var(--transition); position: relative; user-select: none;
  }
  .selector:hover { border-color: var(--accent); }
  .selector-label { color: var(--text-dim); font-size: 11px; }
  .selector-value { color: var(--text); }
  .selector-icon { width: 14px; height: 14px; color: var(--text-dim); }
  .dropdown {
    position: absolute; top: calc(100% + 4px); left: 0;
    background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--radius-sm); padding: 4px; min-width: 100%;
    z-index: 100; display: none; box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }
  .dropdown.open { display: block; }
  .dropdown-item {
    padding: 8px 12px; border-radius: 4px; font-size: 13px;
    cursor: pointer; transition: var(--transition); white-space: nowrap;
  }
  .dropdown-item:hover { background: var(--surface-3); }
  .dropdown-item.selected { color: var(--accent); }
</style>
