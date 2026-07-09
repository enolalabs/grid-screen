<script lang="ts">
  import { onMount } from "svelte";
  import { getSettings, saveSettings } from "../lib/ipc";
  import type { AppSettings } from "../lib/types";

  let settings = $state<AppSettings>({
    auto_start: false,
    default_gap: 4,
    default_margin: 8,
    accent_color: "#7C3AED",
    language: "en",
    first_run_completed: false,
  });
  let saved = $state(false);

  onMount(async () => {
    settings = await getSettings();
  });

  async function handleSave() {
    await saveSettings(settings);
    saved = true;
    setTimeout(() => saved = false, 2000);
  }
</script>

<div class="settings">
  <h2>Settings</h2>

  <label class="setting">
    <span>Auto-start with system</span>
    <input type="checkbox" bind:checked={settings.auto_start} />
  </label>

  <label class="setting">
    <span>Default gap between zones (px)</span>
    <input type="number" bind:value={settings.default_gap} min="0" max="100" />
  </label>

  <label class="setting">
    <span>Default margin from screen edge (px)</span>
    <input type="number" bind:value={settings.default_margin} min="0" max="100" />
  </label>

  <label class="setting">
    <span>Accent color</span>
    <input type="color" bind:value={settings.accent_color} />
  </label>

  <label class="setting">
    <span>Language</span>
    <select bind:value={settings.language}>
      <option value="en">English</option>
      <option value="vi">Tiếng Việt</option>
    </select>
  </label>

  <button onclick={handleSave}>
    {saved ? "Saved!" : "Save Settings"}
  </button>

  <hr />

  <div class="about">
    <h3>About Grid Screen</h3>
    <p>Version 0.1.0</p>
    <p>Cross-platform window zone management.</p>
    <p>Linux (X11) · Windows</p>
  </div>
</div>

<style>
  .settings { display: flex; flex-direction: column; gap: 14px; max-width: 400px; }
  .setting { display: flex; justify-content: space-between; align-items: center; }
  .setting input[type="number"] { width: 70px; padding: 4px 8px; background: #313244; border: 1px solid #45475a; color: #cdd6f4; border-radius: 4px; }
  .setting input[type="checkbox"] { width: 20px; height: 20px; accent-color: #7C3AED; }
  select { padding: 4px 8px; background: #313244; border: 1px solid #45475a; color: #cdd6f4; border-radius: 4px; }
  button { padding: 8px 20px; background: #7C3AED; color: white; border: none; border-radius: 4px; cursor: pointer; align-self: flex-start; }
  hr { width: 100%; border: none; border-top: 1px solid #313244; margin: 8px 0; }
  .about { color: #6c7086; font-size: 13px; }
  .about h3 { color: #cdd6f4; margin: 0 0 8px; }
  .about p { margin: 2px 0; }
</style>
