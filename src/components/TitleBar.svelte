<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  const appWindow = getCurrentWindow();

  function minimize() { appWindow.minimize(); }
  function toggleMaximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<div class="titlebar">
  <div class="traffic-lights">
    <button class="tl tl-close" onclick={close} aria-label="Close"></button>
    <button class="tl tl-min" onclick={minimize} aria-label="Minimize"></button>
    <button class="tl tl-max" onclick={toggleMaximize} aria-label="Maximize"></button>
  </div>
  <span class="title-text">Grid Screen</span>
</div>

<style>
  .titlebar {
    display: flex; align-items: center; height: 44px;
    padding: 0 16px; background: var(--surface);
    border-bottom: 1px solid var(--border); gap: 16px; flex-shrink: 0;
  }
  .traffic-lights { display: flex; gap: 8px; }
  .tl { width: 12px; height: 12px; border-radius: 50%; cursor: pointer; border: none; }
  .tl-close { background: #ff5f57; }
  .tl-min { background: #febc2e; }
  .tl-max { background: #28c840; }
  .title-text {
    font-size: 13px; font-weight: 500; color: var(--text-dim);
    margin-left: auto; margin-right: auto; padding-right: 60px;
  }
</style>
