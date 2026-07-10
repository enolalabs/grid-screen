<script lang="ts">
  interface Props {
    eyebrow: string;
    title: string;
    description: string;
    actionLabel?: string;
    completionLabel?: string;
    onboarding?: boolean;
    onAction?: () => void;
    onCompleteOnboarding?: () => void;
  }

  let {
    eyebrow,
    title,
    description,
    actionLabel,
    completionLabel,
    onboarding = false,
    onAction,
    onCompleteOnboarding,
  }: Props = $props();
</script>

<div class="empty-state">
  <span class="eyebrow">{eyebrow}</span>
  <h3 class="empty-title">{title}</h3>
  <p class="empty-description">{description}</p>

  <div class="empty-actions">
    {#if actionLabel && onAction}
      <button class="btn-action" onclick={onAction}>{actionLabel}</button>
    {/if}
    {#if onboarding && completionLabel && onCompleteOnboarding}
      <button class="btn-complete" onclick={onCompleteOnboarding}>{completionLabel}</button>
    {/if}
  </div>
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .eyebrow {
    display: block;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--primary-bright);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }

  .empty-title {
    margin: 0 0 8px;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .empty-description {
    margin: 0 0 24px;
    font-size: 14px;
    color: var(--text-muted);
    line-height: 1.5;
    max-width: 320px;
  }

  .empty-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 240px;
  }

  .btn-action,
  .btn-complete {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: var(--control-height);
    padding: 0 16px;
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    font-family: var(--sans);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease;
    white-space: nowrap;
  }

  @media (prefers-reduced-motion: reduce) {
    .btn-action,
    .btn-complete {
      transition: none;
    }
  }

  .btn-action {
    background: var(--primary);
    color: #fff;
    border-color: var(--primary);
  }

  .btn-action:hover {
    background: #7c4fe6;
  }

  .btn-action:focus-visible,
  .btn-complete:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .btn-complete {
    background: transparent;
    color: var(--text-muted);
    border-color: var(--border);
  }

  .btn-complete:hover {
    background: var(--surface-3);
    color: var(--text);
  }
</style>
