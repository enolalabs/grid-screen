<script lang="ts">
  interface Props {
    tone: "primary" | "success" | "warning" | "error" | "muted";
    text: string;
  }

  let { tone, text }: Props = $props();
</script>

<span class="badge badge-{tone}" role="status">{text}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 8px;
    border-radius: var(--radius-control);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    line-height: 1;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  .badge-primary {
    background: rgba(139, 92, 246, 0.15);
    color: var(--primary-bright);
    border: 1px solid rgba(139, 92, 246, 0.3);
  }

  .badge-success {
    background: rgba(76, 175, 80, 0.15);
    color: #81c784;
    border: 1px solid rgba(76, 175, 80, 0.3);
  }

  .badge-warning {
    background: rgba(255, 167, 38, 0.15);
    color: #ffb74d;
    border: 1px solid rgba(255, 167, 38, 0.3);
  }

  .badge-error {
    background: rgba(244, 67, 54, 0.15);
    color: #e57373;
    border: 1px solid rgba(244, 67, 54, 0.3);
  }

  .badge-muted {
    background: var(--surface-2);
    color: var(--text-muted);
    border: 1px solid var(--border);
  }
</style>
