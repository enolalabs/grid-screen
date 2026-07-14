<script lang="ts">
  import { settings } from "$lib/stores/settings";

  let showOverlay = $state(true);

  function toggleSnap() {
    settings.update((s) => ({ ...s, snap_enabled: !s.snap_enabled }));
  }

  function toggleOverlay() {
    showOverlay = !showOverlay;
  }
</script>

<div class="detail-section">
  <div class="detail-section-title">Snap</div>
  <div class="toggle-row">
    <span>Enable Snap</span>
    <div
      class="toggle"
      class:on={$settings.snap_enabled}
      onclick={toggleSnap}
      role="switch"
    ></div>
  </div>
  <div class="toggle-row">
    <span>Show Overlay Zones</span>
    <div
      class="toggle"
      class:on={showOverlay}
      onclick={toggleOverlay}
      role="switch"
    ></div>
  </div>
</div>

<style>
  .detail-section { padding: 16px; border-bottom: 1px solid var(--border); }
  .detail-section-title {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.08em; color: var(--text-mute); margin-bottom: 12px;
  }
  .toggle-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 0; font-size: 13px;
  }
  .toggle {
    width: 36px; height: 20px; background: var(--surface-3);
    border-radius: 10px; position: relative; cursor: pointer;
    transition: var(--transition); flex-shrink: 0;
  }
  .toggle::after {
    content: ""; position: absolute; width: 16px; height: 16px;
    border-radius: 50%; background: var(--text-dim); top: 2px; left: 2px;
    transition: var(--transition);
  }
  .toggle.on { background: var(--accent); }
  .toggle.on::after { left: 18px; background: #fff; }
</style>
