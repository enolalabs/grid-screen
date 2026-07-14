<script lang="ts">
  import { settings } from "$lib/stores/settings";
  import { systemStatus } from "$lib/stores/systemStatus";
  import SettingsGroup from "./SettingsGroup.svelte";

  const modifierOptions = ["Shift", "Ctrl", "Alt", "Super"];

  function toggleSnap() {
    settings.update((s) => ({ ...s, snap_enabled: !s.snap_enabled }));
  }

  function setModifier(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    settings.update((s) => ({ ...s, snap_modifier: val }));
  }

  function toggleAutostart() {
    settings.update((s) => ({ ...s, autostart_enabled: !s.autostart_enabled }));
  }

  function toggleTray() {
    settings.update((s) => ({ ...s, minimize_to_tray: !s.minimize_to_tray }));
  }
</script>

<div class="settings-main">
  <SettingsGroup title="Snap Behavior">
    <div class="settings-row">
      <div class="label-group">
        <span>Enable Modifier Snap</span>
        <span class="sub">Hold modifier while dragging a window to snap</span>
      </div>
      <div
        class="toggle"
        class:on={$settings.snap_enabled}
        onclick={toggleSnap}
        role="switch"
      ></div>
    </div>
    <div class="settings-row">
      <div class="label-group">
        <span>Snap Modifier Key</span>
        <span class="sub">Key to hold during drag</span>
      </div>
      <select class="modifier-select" value={$settings.snap_modifier} onchange={setModifier}>
        {#each modifierOptions as opt (opt)}
          <option value={opt}>{opt}</option>
        {/each}
      </select>
    </div>
  </SettingsGroup>

  <SettingsGroup title="Defaults">
    <div class="settings-row">
      <div class="label-group">
        <span>Default Gap</span>
        <span class="sub">Space between zones</span>
      </div>
      <span class="accent-value">{$settings.default_gap_px} px</span>
    </div>
    <div class="settings-row">
      <div class="label-group">
        <span>Default Margin</span>
        <span class="sub">Space around screen edge</span>
      </div>
      <span class="accent-value">{$settings.default_margin_px} px</span>
    </div>
  </SettingsGroup>

  <SettingsGroup title="General">
    <div class="settings-row">
      <div class="label-group">
        <span>Start at Login</span>
        <span class="sub">Launch Grid Screen automatically</span>
      </div>
      <div
        class="toggle"
        class:on={$settings.autostart_enabled}
        onclick={toggleAutostart}
        role="switch"
      ></div>
    </div>
    <div class="settings-row">
      <div class="label-group">
        <span>Minimize to Tray</span>
        <span class="sub">Keep running when window is closed</span>
      </div>
      <div
        class="toggle"
        class:on={$settings.minimize_to_tray}
        onclick={toggleTray}
        role="switch"
      ></div>
    </div>
  </SettingsGroup>

  <SettingsGroup title="System Status">
    <div class="settings-row">
      <span>Session Type</span>
      <span>{$systemStatus.session_type}</span>
    </div>
    <div class="settings-row">
      <span>EWMH / _NET Support</span>
      <span class="status-pill">
        <span class="status-dot"></span>
        {$systemStatus.ewmh_support}
      </span>
    </div>
    <div class="settings-row">
      <span>Window Manager</span>
      <span>{$systemStatus.wm_name}</span>
    </div>
    <div class="settings-row">
      <span>XRandR</span>
      {#if $systemStatus.xrandr_available}
        <span class="status-pill">
          <span class="status-dot"></span>
          Available
        </span>
      {:else}
        <span class="status-pill unavailable">
          <span class="status-dot"></span>
          Unavailable
        </span>
      {/if}
    </div>
    <div class="settings-row">
      <span>Current Workspace</span>
      <span>{$systemStatus.workspace}</span>
    </div>
    <div class="settings-row">
      <span>Connected Screens</span>
      <span>{$systemStatus.connected_screens}</span>
    </div>
  </SettingsGroup>
</div>

<style>
  .settings-main {
    overflow-y: auto;
    padding: 24px;
    max-width: 640px;
    width: 100%;
  }
  .settings-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    font-size: 14px;
  }
  .settings-row:last-child {
    border-bottom: none;
  }
  .settings-row .label-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .settings-row .sub {
    font-size: 12px;
    color: var(--text-dim);
  }
  .modifier-select {
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-family: inherit;
    font-size: 13px;
  }
  .accent-value {
    color: var(--accent);
    font-weight: 500;
  }
  .toggle {
    width: 36px;
    height: 20px;
    background: var(--surface-3);
    border-radius: 10px;
    position: relative;
    cursor: pointer;
    transition: var(--transition);
    flex-shrink: 0;
  }
  .toggle::after {
    content: "";
    position: absolute;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-dim);
    top: 2px;
    left: 2px;
    transition: var(--transition);
  }
  .toggle.on {
    background: var(--accent);
  }
  .toggle.on::after {
    left: 18px;
    background: #fff;
  }
  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 20px;
    font-size: 11px;
    font-weight: 500;
    background: rgba(74, 222, 128, 0.1);
    color: var(--success);
  }
  .status-pill.unavailable {
    background: rgba(248, 113, 113, 0.1);
    color: var(--danger);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--success);
  }
  .status-pill.unavailable .status-dot {
    background: var(--danger);
  }
</style>
