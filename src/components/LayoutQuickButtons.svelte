<script lang="ts">
  import { layouts, selectedLayoutId, sessionOverrides, selectLayout } from "$lib/stores/layout";

  function gridIcon(cols: string, zones: number, spanFirst: boolean | null): string {
    if (cols === "1fr 1fr") return '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="3" x2="12" y2="21"/>';
    if (cols === "1fr 1fr 1fr") return '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/>';
    if (cols === "3fr 1fr") return '<rect x="3" y="3" width="13" height="18" rx="2"/><rect x="18" y="3" width="3" height="18" rx="1"/>';
    if (cols === "2fr 1fr" && spanFirst) return '<rect x="3" y="3" width="12" height="18" rx="2"/><rect x="17" y="3" width="4" height="8" rx="1"/><rect x="17" y="13" width="4" height="8" rx="1"/>';
    if (cols === "1fr 2fr 1fr") return '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/>';
    return '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="3" x2="12" y2="21"/>';
  }
</script>

<div class="layout-quick">
  {#each $layouts as l (l.id)}
    <div
      class="layout-btn"
      class:active={l.id === $selectedLayoutId}
      title={l.name}
      onclick={() => selectLayout(l.id)}
      role="button"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {@html gridIcon(l.columns, l.zones, l.span_first)}
      </svg>
    </div>
  {/each}
</div>

<style>
  .layout-quick { display: flex; gap: 6px; margin-left: auto; }
  .layout-btn {
    width: 36px; height: 36px; border-radius: var(--radius-sm);
    background: var(--surface); border: 1px solid var(--border);
    display: flex; align-items: center; justify-content: center;
    cursor: pointer; transition: var(--transition);
  }
  .layout-btn:hover { border-color: var(--accent); }
  .layout-btn.active { border-color: var(--accent); background: var(--accent-soft); }
  .layout-btn svg { width: 18px; height: 18px; color: var(--text-dim); }
  .layout-btn.active svg { color: var(--accent); }
</style>
