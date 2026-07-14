<script lang="ts">
  import { systemStatus } from "$lib/stores/systemStatus";
</script>

<div class="detail-section">
  <div class="detail-section-title">System Status</div>
  <div class="status-row">
    <span class="lbl">Session</span>
    <span>{$systemStatus.session_type}</span>
  </div>
  <div class="status-row">
    <span class="lbl">EWMH Support</span>
    <span class="status-pill">
      <span class="status-dot"></span>
      {$systemStatus.ewmh_support}
    </span>
  </div>
  <div class="status-row">
    <span class="lbl">Workspace</span>
    <span>{$systemStatus.workspace}</span>
  </div>
  <div class="status-row">
    <span class="lbl">Active Screen</span>
    <span>{$systemStatus.connected_screens}</span>
  </div>
  {#each $systemStatus.errors as err (err)}
    <div class="status-row error-row">
      <span class="lbl">Error</span>
      <span class="error-text">{err}</span>
    </div>
  {/each}
</div>

<style>
  .detail-section { padding: 16px; border-bottom: 1px solid var(--border); }
  .detail-section-title {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.08em; color: var(--text-mute); margin-bottom: 12px;
  }
  .status-row {
    display: flex; justify-content: space-between;
    font-size: 12px; padding: 4px 0;
  }
  .status-row .lbl { color: var(--text-dim); }
  .status-pill {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 4px 10px; border-radius: 20px; font-size: 11px; font-weight: 500;
    background: rgba(74, 222, 128, 0.1); color: var(--success);
  }
  .status-dot {
    width: 6px; height: 6px; border-radius: 50%; background: var(--success);
  }
  .error-row { color: var(--danger); }
  .error-text { color: var(--danger); }
</style>
