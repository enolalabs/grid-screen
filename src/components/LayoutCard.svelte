<script lang="ts">
  import type { Layout } from "$lib/shared-types";

  interface Props {
    layout: Layout;
    isActive: boolean;
    onSelect: () => void;
    onEdit: () => void;
    onDelete: () => void;
  }

  let { layout, isActive, onSelect, onEdit, onDelete }: Props = $props();

  const isSaved = $derived(layout.type === "saved");

  function previewGridStyle(): string {
    const cols = layout.columns.split(/\s+/).length;
    const rows = layout.span_first ? 2 : 1;
    return `grid-template-columns: repeat(${cols}, 1fr); grid-template-rows: repeat(${rows}, 1fr);`;
  }
</script>

<div
  class="layout-card"
  class:active={isActive}
  onclick={onSelect}
  role="button"
  tabindex="-1"
>
  <div class="layout-card-preview" style={previewGridStyle()}>
    {#each Array.from({ length: layout.zones }, (_, i) => i) as i (i)}
      <div></div>
    {/each}
  </div>
  <div class="layout-card-name">{layout.name}</div>
  <div class="layout-card-meta">
    {layout.zones} {layout.zones === 1 ? "zone" : "zones"}
    &middot; {layout.type}
  </div>
  <div class="layout-card-actions">
    <button
      class="layout-action-btn"
      onclick={(e: Event) => {
        e.stopPropagation();
        onSelect();
      }}
    >
      Select
    </button>
    {#if isSaved}
      <button
        class="layout-action-btn"
        onclick={(e: Event) => {
          e.stopPropagation();
          onEdit();
        }}
      >
        Edit
      </button>
      <button
        class="layout-action-btn"
        onclick={(e: Event) => {
          e.stopPropagation();
          onDelete();
        }}
      >
        Delete
      </button>
    {/if}
  </div>
</div>

<style>
  .layout-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 20px;
    cursor: pointer;
    transition: var(--transition);
  }
  .layout-card:hover {
    border-color: var(--accent);
    transform: translateY(-2px);
  }
  .layout-card.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .layout-card-preview {
    aspect-ratio: 16/10;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    display: grid;
    gap: 4px;
    padding: 6px;
    margin-bottom: 14px;
  }
  .layout-card-preview > div {
    background: var(--surface-3);
    border-radius: 3px;
  }
  .layout-card-name {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 4px;
  }
  .layout-card-meta {
    font-size: 11px;
    color: var(--text-mute);
  }
  .layout-card-actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }
  .layout-action-btn {
    padding: 4px 10px;
    font-size: 11px;
    border-radius: 4px;
    background: var(--surface-3);
    border: 1px solid var(--border);
    color: var(--text-dim);
    cursor: pointer;
    transition: var(--transition);
    font-family: inherit;
  }
  .layout-action-btn:hover {
    color: var(--accent);
    border-color: var(--accent);
  }
</style>
