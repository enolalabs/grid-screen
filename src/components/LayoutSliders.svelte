<script lang="ts">
  import { selectedLayout, sessionOverrides } from "$lib/stores/layout";

  let ratio = $state(50);
  let gap = $state(10);
  let margin = $state(16);

  let ratioTimer: ReturnType<typeof setTimeout> | null = null;
  let gapTimer: ReturnType<typeof setTimeout> | null = null;
  let marginTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if ($selectedLayout) {
      ratio = $selectedLayout.ratio ?? 50;
      gap = $selectedLayout.gap_px;
      margin = $selectedLayout.margin_px;
    }
  });

  const showDividerRatio = $derived(
    $selectedLayout !== null && $selectedLayout.zones === 2 && $selectedLayout.ratio !== null
  );

  function debouncedUpdate() {
    if (ratioTimer) clearTimeout(ratioTimer);
    if (gapTimer) clearTimeout(gapTimer);
    if (marginTimer) clearTimeout(marginTimer);
    ratioTimer = setTimeout(applyOverrides, 150);
    gapTimer = setTimeout(applyOverrides, 150);
    marginTimer = setTimeout(applyOverrides, 150);
  }

  function applyOverrides() {
    sessionOverrides.set({
      ratio,
      gap_px: gap,
      margin_px: margin,
    });
  }

  function handleRatioInput(e: Event) {
    ratio = Number((e.target as HTMLInputElement).value);
    debouncedUpdate();
  }

  function handleGapInput(e: Event) {
    gap = Number((e.target as HTMLInputElement).value);
    debouncedUpdate();
  }

  function handleMarginInput(e: Event) {
    margin = Number((e.target as HTMLInputElement).value);
    debouncedUpdate();
  }
</script>

<div class="detail-section">
  <div class="detail-section-title">
    {#if $selectedLayout}
      Layout: {$selectedLayout.name}
    {:else}
      No layout selected
    {/if}
  </div>
  {#if showDividerRatio}
    <div class="slider-group">
      <div class="slider-label"><span>Divider Ratio</span><span>{ratio}%</span></div>
      <input type="range" class="slider" min="10" max="90" value={ratio} oninput={handleRatioInput} />
    </div>
  {/if}
  <div class="slider-group">
    <div class="slider-label"><span>Gap</span><span>{gap} px</span></div>
    <input type="range" class="slider" min="0" max="40" value={gap} oninput={handleGapInput} />
  </div>
  <div class="slider-group">
    <div class="slider-label"><span>Margin</span><span>{margin} px</span></div>
    <input type="range" class="slider" min="0" max="60" value={margin} oninput={handleMarginInput} />
  </div>
</div>

<style>
  .detail-section { padding: 16px; border-bottom: 1px solid var(--border); }
  .detail-section-title {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.08em; color: var(--text-mute); margin-bottom: 12px;
  }
  .slider-group { margin-bottom: 14px; }
  .slider-label {
    display: flex; justify-content: space-between;
    font-size: 13px; margin-bottom: 6px;
  }
  .slider-label span:last-child { color: var(--accent); font-weight: 500; }
  .slider {
    width: 100%; height: 4px; -webkit-appearance: none;
    background: var(--surface-3); border-radius: 2px; outline: none;
  }
  .slider::-webkit-slider-thumb {
    -webkit-appearance: none; width: 14px; height: 14px; border-radius: 50%;
    background: var(--accent); cursor: pointer; border: 2px solid var(--bg);
  }
</style>
