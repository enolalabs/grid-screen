<script lang="ts">
  interface Props {
    label: string;
    value: string;
    tone?: "primary" | "success" | "warning" | "error" | "muted";
  }

  let { label, value, tone = "muted" }: Props = $props();
</script>

<div class="status-row" class:has-tone={tone !== "muted"}>
  <span class="status-label">{label}</span>
  <span class="status-value tone-{tone}">{value}</span>
</div>

<style>
  .status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 0;
  }

  .status-label {
    font-size: 13px;
    color: var(--text-muted);
  }

  .status-value {
    font-family: var(--mono);
    font-size: 13px;
    color: var(--text-muted);
    text-align: right;
  }

  .tone-primary { color: var(--primary-bright); }
  .tone-success { color: #81c784; }
  .tone-warning { color: #ffb74d; }
  .tone-error { color: #e57373; }
  .tone-muted { color: var(--text-muted); }
</style>
