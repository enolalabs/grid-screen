<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentState, listLayouts, getSettings } from "./lib/ipc";
  import { currentState, savedLayouts, settings } from "./lib/stores";
  import LayoutEditor from "./routes/LayoutEditor.svelte";
  import LayoutManager from "./routes/LayoutManager.svelte";
  import Settings from "./routes/Settings.svelte";

  let activeTab = $state<"editor" | "layouts" | "settings">("editor");

  onMount(async () => {
    const state = await getCurrentState();
    currentState.set(state);
    const layouts = await listLayouts();
    savedLayouts.set(layouts);
    const s = await getSettings();
    settings.set(s);
  });
</script>

<div class="app-shell">
  <nav class="tab-bar">
    <button class:active={activeTab === "editor"} onclick={() => activeTab = "editor"}>Editor</button>
    <button class:active={activeTab === "layouts"} onclick={() => activeTab = "layouts"}>Layouts</button>
    <button class:active={activeTab === "settings"} onclick={() => activeTab = "settings"}>Settings</button>
  </nav>
  <main class="content">
    {#if activeTab === "editor"}
      <LayoutEditor />
    {:else if activeTab === "layouts"}
      <LayoutManager />
    {:else}
      <Settings />
    {/if}
  </main>
</div>

<style>
  .app-shell { display: flex; flex-direction: column; height: 100vh; font-family: system-ui; }
  .tab-bar { display: flex; gap: 4px; padding: 8px 12px; background: #1e1e2e; border-bottom: 1px solid #313244; }
  .tab-bar button { padding: 6px 16px; border: none; background: transparent; color: #cdd6f4; cursor: pointer; border-radius: 4px; }
  .tab-bar button.active { background: #7C3AED; color: white; }
  .content { flex: 1; overflow: auto; padding: 16px; background: #181825; color: #cdd6f4; }
</style>
