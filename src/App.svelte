<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import TabNav from "./components/TabNav.svelte";
  import ArrangeView from "./components/ArrangeView.svelte";
  import LayoutsView from "./components/LayoutsView.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import ToastContainer from "./components/ToastContainer.svelte";
  import ArrangeStateOverlay from "./components/ArrangeStateOverlay.svelte";
  import { commands } from "./lib/commands";
  import { registerEventListeners } from "./lib/events";
  import { layouts, selectedLayoutId, selectLayout } from "./lib/stores/layout";
  import { screens, selectedScreenId } from "./lib/stores/screen";
  import { windows } from "./lib/stores/windows";
  import { settings } from "./lib/stores/settings";
  import { systemStatus } from "./lib/stores/systemStatus";

  let activeTab: "arrange" | "layouts" | "settings" = $state("arrange");
  let loading = $state(true);
  let error = $state<string | null>(null);
  let unregisterEvents: (() => void) | undefined;

  onMount(async () => {
    try {
      const data = await commands.bootstrap();
      screens.set(data.screens);
      layouts.set(data.layouts);
      windows.set(data.windows);
      settings.set(data.settings);
      systemStatus.set(data.system_status);

      if (data.screens.length > 0) {
        selectedScreenId.set(data.screens[0].id);
      }
      if (data.settings.last_layout_id) {
        selectLayout(data.settings.last_layout_id);
      } else if (data.layouts.length > 0) {
        selectLayout(data.layouts[0].id);
      }

      unregisterEvents = registerEventListeners();
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  });

  onDestroy(() => {
    unregisterEvents?.();
  });
</script>

<div class="app">
  <TitleBar />
  <TabNav {activeTab} on:tabChange={(e) => activeTab = e.detail} />
  <div class="view-container">
    {#if loading}
      <div class="loading">Loading...</div>
    {:else if error}
      <div class="error-state">
        <p>Failed to start: {error}</p>
        <button onclick={() => location.reload()}>Retry</button>
      </div>
    {:else}
      {#if activeTab === "arrange"}
        <ArrangeView />
      {:else if activeTab === "layouts"}
        <LayoutsView />
      {:else}
        <SettingsView />
      {/if}
    {/if}
  </div>
  <ToastContainer />
  <ArrangeStateOverlay />
</div>

<style>
  .app {
    display: flex; flex-direction: column;
    height: 100vh; background: var(--bg);
    color: var(--text); font-family: 'Inter', sans-serif;
  }
  .view-container { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
  .loading, .error-state {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; height: 100%; color: var(--text-dim);
    font-size: 14px; gap: 12px;
  }
  .error-state button {
    padding: 8px 20px; background: var(--accent); color: #fff;
    border: none; border-radius: var(--radius-sm); cursor: pointer;
    font-family: inherit; font-size: 13px;
  }
</style>
