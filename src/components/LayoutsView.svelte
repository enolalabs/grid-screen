<script lang="ts">
  import { layouts, selectedLayoutId, selectLayout } from "$lib/stores/layout";
  import { commands } from "$lib/commands";
  import { showToast } from "$lib/stores/toasts";
  import LayoutCard from "./LayoutCard.svelte";
  import NewLayoutModal from "./NewLayoutModal.svelte";
  import type { Layout } from "$lib/shared-types";

  let showNewLayoutModal = $state(false);

  function handleSelect(layout: Layout) {
    selectLayout(layout.id);
  }

  function handleEdit(layout: Layout) {
    selectLayout(layout.id);
  }

  async function handleDelete(layout: Layout) {
    try {
      await commands.deleteLayout(layout.id);
      if ($selectedLayoutId === layout.id) {
        selectLayout("");
      }
      await loadLayouts();
      showToast("Layout deleted", "success");
    } catch (e) {
      showToast(String(e), "error");
    }
  }

  async function loadLayouts() {
    try {
      const data = await commands.bootstrap();
      layouts.set(data.layouts);
    } catch (e) {
      showToast(String(e), "error");
    }
  }

  async function handleSaveLayout(layout: Layout) {
    try {
      await commands.saveLayout(layout);
      await loadLayouts();
      selectLayout(layout.id);
      showToast("Layout saved", "success");
    } catch (e) {
      showToast(String(e), "error");
    }
  }
</script>

<div class="view">
  <div class="layouts-header">
    <h2>Layouts</h2>
    <button class="btn-new-layout" onclick={() => (showNewLayoutModal = true)}>
      + New Layout
    </button>
  </div>
  <div class="layouts-grid">
    {#each $layouts as layout (layout.id)}
      <LayoutCard
        {layout}
        isActive={layout.id === $selectedLayoutId}
        onSelect={() => handleSelect(layout)}
        onEdit={() => handleEdit(layout)}
        onDelete={() => handleDelete(layout)}
      />
    {/each}
  </div>
</div>

{#if showNewLayoutModal}
  <NewLayoutModal
    onClose={() => (showNewLayoutModal = false)}
    onSave={(l: Layout) => {
      handleSaveLayout(l);
      showNewLayoutModal = false;
    }}
  />
{/if}

<style>
  .view {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }
  .layouts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
  }
  .layouts-header h2 {
    font-size: 18px;
    font-weight: 700;
  }
  .btn-new-layout {
    padding: 10px 20px;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    font-family: inherit;
    transition: var(--transition);
  }
  .btn-new-layout:hover {
    transform: translateY(-1px);
  }
  .layouts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
    padding: 24px;
    overflow-y: auto;
  }
</style>
