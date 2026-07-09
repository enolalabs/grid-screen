<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentState, getSettings } from "./lib/ipc";
  import { currentState, savedLayouts, settings } from "./lib/stores";
  import { notifications, notify } from "./lib/notifications";
  import LayoutEditor from "./routes/LayoutEditor.svelte";
  import LayoutManager from "./routes/LayoutManager.svelte";
  import Settings from "./routes/Settings.svelte";
  import { listen } from "@tauri-apps/api/event";

  let activeTab = $state<"editor" | "layouts" | "settings">("editor");
  let showOnboarding = $state(false);
  let notifs = $state<Array<{id: string, message: string, type: string}>>([]);

  onMount(async () => {
    try {
      const state = await getCurrentState();
      currentState.set(state);
      savedLayouts.set(state.saved_layouts);
      settings.set(state.settings);

      if (!state.settings.first_run_completed) {
        showOnboarding = true;
      }
    } catch (e) {
      console.error("Failed to load state:", e);
    }

    const unsubNotifs = notifications.subscribe(n => notifs = n);

    const unlistenEvent = await listen<{level: string, message: string}>("user-notification", (event) => {
      const { level, message } = event.payload;
      notify(message, level as "info" | "warning" | "error");
    });

    return () => {
      unsubNotifs();
      unlistenEvent();
    };
  });

  function dismissOnboarding() { showOnboarding = false; }
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

{#if showOnboarding}
  <div class="onboarding-overlay" role="dialog" aria-label="First-run guide">
    <div class="onboarding-card">
      <h3>Welcome to Grid Screen</h3>
      <p>Drag on a monitor to create your first zone.</p>
      <p>Then drag any application window into a zone to snap it into place.</p>
      <button onclick={dismissOnboarding}>Got it</button>
    </div>
  </div>
{/if}

<div class="toast-container" role="status" aria-live="polite">
  {#each notifs as n (n.id)}
    <div class="toast toast-{n.type}">{n.message}</div>
  {/each}
</div>

<style>
  .app-shell { display: flex; flex-direction: column; height: 100vh; font-family: system-ui; }
  .tab-bar { display: flex; gap: 4px; padding: 8px 12px; background: #1e1e2e; border-bottom: 1px solid #313244; }
  .tab-bar button { padding: 6px 16px; border: none; background: transparent; color: #cdd6f4; cursor: pointer; border-radius: 4px; }
  .tab-bar button.active { background: #7C3AED; color: white; }
  .content { flex: 1; overflow: auto; padding: 16px; background: #181825; color: #cdd6f4; }
  .onboarding-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .onboarding-card { background: #1e1e2e; padding: 24px 32px; border-radius: 12px; max-width: 400px; text-align: center; }
  .onboarding-card button { margin-top: 16px; padding: 8px 24px; background: #7C3AED; color: white; border: none; border-radius: 6px; cursor: pointer; }
  .toast-container { position: fixed; bottom: 16px; right: 16px; display: flex; flex-direction: column; gap: 8px; z-index: 200; }
  .toast { padding: 10px 20px; border-radius: 6px; font-size: 14px; animation: slideIn 0.3s ease; }
  .toast-info { background: #313244; color: #cdd6f4; }
  .toast-warning { background: #f9e2af; color: #1e1e2e; }
  .toast-error { background: #f38ba8; color: #1e1e2e; }
  @keyframes slideIn { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>
