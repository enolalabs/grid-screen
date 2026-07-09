<script lang="ts">
  import { onMount } from "svelte";
  import { savedLayouts } from "../lib/stores";
  import { listLayouts, deleteLayout, setDefaultLayout } from "../lib/ipc";
  import type { SavedLayout } from "../lib/types";

  let layouts = $state<SavedLayout[]>([]);

  onMount(async () => {
    layouts = await listLayouts();
  });

  async function handleDelete(id: string, name: string) {
    if (confirm(`Delete layout "${name}"?`)) {
      await deleteLayout(id);
      layouts = await listLayouts();
    }
  }
</script>

<div class="layout-manager">
  <h2>Saved Layouts</h2>
  {#if layouts.length === 0}
    <p class="empty">No layouts saved yet. Create one in the Editor tab.</p>
  {:else}
    <div class="layout-list">
      {#each layouts as layout (layout.id)}
        <div class="layout-card" role="listitem">
          <div class="layout-info">
            <strong>{layout.name}</strong>
            <span class="layout-meta">{layout.zones.length} zones</span>
          </div>
          <div class="layout-actions">
            <button onclick={() => setDefaultLayout(layout.id)}>Set Default</button>
            <button class="danger" onclick={() => handleDelete(layout.id, layout.name)}>Delete</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .layout-manager { display: flex; flex-direction: column; gap: 12px; }
  .empty { color: #6c7086; }
  .layout-list { display: flex; flex-direction: column; gap: 8px; }
  .layout-card { display: flex; justify-content: space-between; align-items: center; padding: 12px; background: #1e1e2e; border-radius: 8px; border: 1px solid #313244; }
  .layout-info { display: flex; flex-direction: column; gap: 2px; }
  .layout-meta { font-size: 12px; color: #6c7086; }
  .layout-actions { display: flex; gap: 6px; }
  button { padding: 6px 14px; border: none; border-radius: 4px; cursor: pointer; background: #45475a; color: #cdd6f4; }
  button.danger { background: #f38ba8; color: #1e1e2e; }
</style>
