import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/svelte/svelte5";
import LayoutThumbnail from "../components/LayoutThumbnail.svelte";
import AppShell from "../components/AppShell.svelte";
import type { FrontendState } from "../types";
import { currentState } from "../stores";

afterEach(() => {
  cleanup();
  currentState.set(null);
});

function makeState(overrides: Partial<FrontendState> = {}): FrontendState {
  return {
    monitors: [
      { id: "m1", name: "<img src=x onerror=alert(1)>", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ],
    active_layouts: [],
    saved_layouts: [
      {
        id: "xss-layout",
        name: "<img src=x onerror=alert(1)>",
        arrangement_id: "a1",
        zones: [
          { id: "z1", name: "<script>document.cookie</script>", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 },
        ],
        monitor_id: "m1",
      },
    ],
    is_paused: false,
    settings: {
      auto_start: false,
      default_gap: 4,
      default_margin: 8,
      accent_color: "#8B5CF6",
      language: "en",
      first_run_completed: true,
      default_layout_id: null,
    },
    ...overrides,
  };
}

function renderLayoutThumbnail(props: {
  layoutId: string;
  zones: Array<{ id: string; name: string; x: number; y: number; width: number; height: number; gap: number; margin: number }>;
  label: string;
}) {
  return render(LayoutThumbnail, { props });
}

describe("Security rendering", () => {
  it("renders malicious layout name as escaped text, not injected HTML", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }],
      label: "<img src=x onerror=alert(1)>",
    });

    expect(view.container.querySelector("img")).toBeNull();
    expect(view.container.textContent).toContain("<img src=x onerror=alert(1)>");
  });

  it("renders malicious zone name without injecting HTML", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "<script>document.cookie</script>", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }],
      label: "Safe Layout",
    });

    expect(view.container.querySelector("script")).toBeNull();
    expect(view.container.querySelector("img")).toBeNull();

    const labelSpan = view.container.querySelector(".thumbnail-label");
    expect(labelSpan?.textContent).toBe("Safe Layout");
  });

  it("renders malicious settings/notification content as text, not HTML", () => {
    const view = render(AppShell, {
      props: {
        activeView: "settings",
        initialization: {
          status: "loaded",
          state: makeState({
            settings: {
              auto_start: false,
              default_gap: 4,
              default_margin: 8,
              accent_color: "#8B5CF6",
              language: "en",
              first_run_completed: true,
              default_layout_id: null,
            },
          }),
        },
        isPaused: false,
        monitorCount: 1,
        onNavigate: () => {},
        onRetry: () => {},
        children: "Settings Content",
      },
    });

    expect(view.container.querySelector("img")).toBeNull();
  });

  it("does not use {@html} in any runtime source", () => {
    const maliciousLabel = "<img src=x onerror=alert(1)>";
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Safe", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }],
      label: maliciousLabel,
    });

    const imgEl = view.container.querySelector("img");
    expect(imgEl).toBeNull();

    const scripts = view.container.querySelectorAll("script");
    expect(scripts.length).toBe(0);

    const labelSpan = view.container.querySelector(".thumbnail-label");
    expect(labelSpan?.textContent).toBe(maliciousLabel);
  });
});

describe("Thumbnail numeric clamping", () => {
  it("clamps zone coordinates when NaN", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "NaN zone", x: NaN, y: 0, width: 1, height: 1, gap: 4, margin: 8 }],
      label: "NaN Test",
    });

    const zoneEl = view.container.querySelector(".thumbnail-zone") as HTMLElement;
    expect(zoneEl).toBeTruthy();
    const style = zoneEl.getAttribute("style") ?? "";
    // Browsers ignore NaN% in CSS; we assert no "NaN" in style attr
    expect(style).not.toContain("NaN%");
  });

  it("clamps zone coordinates when negative", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Neg zone", x: -0.5, y: -0.5, width: 2, height: 2, gap: 4, margin: 8 }],
      label: "Negative Margin",
    });

    const zoneEl = view.container.querySelector(".thumbnail-zone") as HTMLElement;
    expect(zoneEl).toBeTruthy();
    const style = zoneEl.getAttribute("style") ?? "";
    // Should not contain negative percentages beyond the container (browser clips overflow)
    expect(style).not.toContain("-0%");
  });

  it("omits non-finite Infinity attributes from SVG geometry", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Inf zone", x: 0, y: 0, width: Infinity, height: -Infinity, gap: 4, margin: 8 }],
      label: "Infinity Test",
    });

    const zoneEl = view.container.querySelector(".thumbnail-zone") as HTMLElement;
    expect(zoneEl).toBeTruthy();
    const style = zoneEl.getAttribute("style") ?? "";
    expect(style).not.toContain("Infinity");
    expect(style).not.toContain("-Infinity");
  });

  it("clamps values above 1.0 in thumbnail", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Over", x: 0, y: 0, width: 1.5, height: 1.5, gap: 4, margin: 8 }],
      label: "Oversize",
    });

    const zoneEl = view.container.querySelector(".thumbnail-zone") as HTMLElement;
    expect(zoneEl).toBeTruthy();
    const style = zoneEl.getAttribute("style") ?? "";
    expect(style).not.toContain("150%");
  });

  it("clamps zero width/height to at least a visible minimum", () => {
    const view = renderLayoutThumbnail({
      layoutId: "l1",
      zones: [{ id: "z1", name: "Zero", x: 0, y: 0, width: 0, height: 0, gap: 4, margin: 8 }],
      label: "Zero Test",
    });

    const zoneEl = view.container.querySelector(".thumbnail-zone") as HTMLElement;
    expect(zoneEl).toBeTruthy();
    const style = zoneEl.getAttribute("style") ?? "";
    // Width and height should be clamped above 0, not 0%
    expect(style).toMatch(/width:\s*(?!0%)\d/);
    expect(style).toMatch(/height:\s*(?!0%)\d/);
  });
});
