import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/svelte/svelte5";
import LayoutEditor from "../LayoutEditor.svelte";

vi.mock("../../lib/ipc", () => ({
  applyLayout: vi.fn().mockResolvedValue(undefined),
  saveLayout: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../lib/stores", () => {
  const { writable } = require("svelte/store");
  return {
    currentState: writable({
      monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
      active_layouts: [], saved_layouts: [], is_paused: false,
      settings: { default_gap: 4, default_margin: 8, accent_color: "#7C3AED", language: "en", auto_start: false, first_run_completed: true },
    }),
    selectedMonitor: writable(null),
    activeLayout: writable(null),
    savedLayouts: writable([]),
    settings: writable(null),
  };
});

describe("LayoutEditor", () => {
  it("renders monitor name and resolution", async () => {
    const { findByText } = render(LayoutEditor);
    expect(await findByText("Main (1920×1080)")).toBeTruthy();
  });

  it("creates zone on canvas click", async () => {
    const { container, findByRole } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 400, clientY: 200 });
    expect(await findByRole("region")).toBeTruthy();
  });

  it("shows styled confirmation dialog on right-click delete", async () => {
    const { container, findByRole } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 200, clientY: 100 });
    const zone = await findByRole("region");
    await fireEvent.contextMenu(zone);
    expect(await findByRole("alertdialog")).toBeTruthy();
  });

  it("moves zone with arrow keys", async () => {
    const { container, findByRole } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 100, clientY: 100 });
    const zone = await findByRole("region");
    (zone as HTMLElement).focus();
    await fireEvent.keyDown(zone, { key: "ArrowRight" });
  });
});
