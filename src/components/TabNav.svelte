<script lang="ts">
  export let activeTab: "arrange" | "layouts" | "settings" = "arrange";

  const tabs = [
    {
      id: "arrange" as const,
      label: "Arrange",
      icon: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    },
    {
      id: "layouts" as const,
      label: "Layouts",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/>',
    },
    {
      id: "settings" as const,
      label: "Settings",
      icon: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    },
  ];

  function handleTabClick(id: string) {
    activeTab = id as "arrange" | "layouts" | "settings";
    emit("tabChange", activeTab);
  }

  import { createEventDispatcher } from "svelte";
  const emit = createEventDispatcher<{ tabChange: string }>();

  $: svgContent = {
    arrange: tabs[0].icon,
    layouts: tabs[1].icon,
    settings: tabs[2].icon,
  };
</script>

<div class="nav-tabs">
  {#each tabs as { id, label, icon }}
    <button
      class="nav-tab"
      class:active={activeTab === id}
      onclick={() => handleTabClick(id)}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {@html icon}
      </svg>
      {label}
    </button>
  {/each}
</div>

<style>
  .nav-tabs {
    display: flex; gap: 0; padding: 0 16px;
    background: var(--surface); border-bottom: 1px solid var(--border);
    height: 44px; flex-shrink: 0;
  }
  .nav-tab {
    display: flex; align-items: center; gap: 8px;
    padding: 0 16px; font-size: 13px; font-weight: 500;
    color: var(--text-dim); cursor: pointer;
    border: none; background: none; font-family: inherit;
    border-bottom: 2px solid transparent;
    transition: var(--transition); position: relative; top: 1px;
    user-select: none;
  }
  .nav-tab:hover { color: var(--text); }
  .nav-tab.active { color: var(--accent); border-bottom-color: var(--accent); }
</style>
