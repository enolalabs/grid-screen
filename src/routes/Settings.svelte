<script lang="ts">
  import { onMount } from "svelte";
  import { getSettings, saveSettings, getCurrentState } from "../lib/ipc";
  import { currentState, savedLayouts, settings as settingsStore } from "../lib/stores";
  import { notify } from "../lib/notifications";
  import type { AppSettings } from "../lib/types";
  import Panel from "../lib/components/Panel.svelte";
  import Button from "../lib/components/Button.svelte";
  import ErrorPanel from "../lib/components/ErrorPanel.svelte";

  let settings = $state<AppSettings>({
    auto_start: false,
    default_gap: 4,
    default_margin: 8,
    accent_color: "#7C3AED",
    language: "en",
    first_run_completed: false,
    default_layout_id: null,
  });
  let pending = $state(false);
  let saveError = $state<string | null>(null);
  let saved = $state(false);

  onMount(async () => {
    settings = await getSettings();
  });

  async function handleSave() {
    if (pending) return;
    pending = true;
    saveError = null;
    saved = false;
    try {
      await saveSettings(settings);
      const state = await getCurrentState();
      currentState.set(state);
      settingsStore.set(state.settings);
      savedLayouts.set(state.saved_layouts);
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      saveError = message;
      notify("Failed to save settings", "error");
    } finally {
      pending = false;
    }
  }
</script>

<div class="settings">
  <h2 class="settings-heading">Settings</h2>

  <div class="settings-sections">
    <Panel title="Runtime">
      <label class="setting">
        <span class="setting-label">Auto-start with system</span>
        <input type="checkbox" bind:checked={settings.auto_start} />
      </label>
    </Panel>

    <Panel title="Layout Defaults">
      <label class="setting">
        <span class="setting-label">Default gap between zones (px)</span>
        <input type="number" bind:value={settings.default_gap} min="0" max="100" />
      </label>
      <label class="setting">
        <span class="setting-label">Default margin from screen edge (px)</span>
        <input type="number" bind:value={settings.default_margin} min="0" max="100" />
      </label>
      <label class="setting">
        <span class="setting-label">Default layout ID</span>
        <input type="text" bind:value={settings.default_layout_id} placeholder="None" />
      </label>
    </Panel>

    <Panel title="Appearance">
      <label class="setting">
        <span class="setting-label">Accent color</span>
        <input type="color" bind:value={settings.accent_color} />
      </label>
    </Panel>

    <Panel title="Language">
      <label class="setting">
        <span class="setting-label">Language</span>
        <select bind:value={settings.language}>
          <option value="en">English</option>
          <option value="vi">Tiếng Việt</option>
        </select>
      </label>
    </Panel>

    <Panel title="About">
      <div class="about">
        <h3>About Grid Screen</h3>
        <p>Version 0.1.0</p>
        <p>Cross-platform window zone management.</p>
        <p>Linux (X11) · Windows</p>
      </div>
    </Panel>
  </div>

  <div class="save-area">
    <Button variant="primary" disabled={pending} onclick={handleSave}>
      {saved ? "Saved!" : "Save Settings"}
    </Button>
    {#if saved}
      <span class="save-feedback">Settings saved successfully</span>
    {/if}
  </div>

  {#if saveError}
    <ErrorPanel title="Save Failed" message={saveError} />
  {/if}
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .settings-heading {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .setting {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
  }

  .setting-label {
    font-size: 13px;
    color: var(--text);
    flex: 1;
  }

  .setting input[type="number"] {
    width: 70px;
    padding: 4px 8px;
    background: var(--surface-1);
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: var(--radius-control);
    font-family: var(--mono);
    font-size: 13px;
  }

  .setting input[type="text"] {
    width: 160px;
    padding: 4px 8px;
    background: var(--surface-1);
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: var(--radius-control);
    font-size: 13px;
  }

  .setting input[type="checkbox"] {
    width: 20px;
    height: 20px;
    accent-color: var(--primary);
  }

  .setting input[type="color"] {
    width: 36px;
    height: 28px;
    padding: 2px;
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    cursor: pointer;
  }

  select {
    padding: 4px 8px;
    background: var(--surface-1);
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: var(--radius-control);
    font-size: 13px;
  }

  .save-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
  }

  .save-feedback {
    font-size: 13px;
    color: #81c784;
  }

  .about {
    color: var(--text-muted);
    font-size: 13px;
  }

  .about h3 {
    color: var(--text);
    margin: 0 0 8px;
  }

  .about p {
    margin: 2px 0;
  }
</style>
