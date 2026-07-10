import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, waitFor, cleanup } from "@testing-library/svelte/svelte5";
import LayoutEditor from "../LayoutEditor.svelte";
import { applyLayout, saveLayout, getCurrentState } from "../../lib/ipc";
import { currentState, savedLayouts } from "../../lib/stores";

vi.mock("../../lib/ipc", () => ({
  applyLayout: vi.fn().mockResolvedValue(undefined),
  saveLayout: vi.fn().mockResolvedValue(undefined),
  getCurrentState: vi.fn().mockResolvedValue({
    monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
    active_layouts: [],
    saved_layouts: [],
    is_paused: false,
    settings: {
      default_gap: 4,
      default_margin: 8,
      accent_color: "#8b5cf6",
      language: "en",
      auto_start: false,
      first_run_completed: true,
      default_layout_id: null,
    },
  }),
  listLayouts: vi.fn().mockResolvedValue([]),
  deleteLayout: vi.fn().mockResolvedValue(undefined),
  togglePause: vi.fn().mockResolvedValue(false),
  getSettings: vi.fn().mockResolvedValue({
    default_gap: 4,
    default_margin: 8,
    accent_color: "#8b5cf6",
    language: "en",
    auto_start: false,
    first_run_completed: true,
    default_layout_id: null,
  }),
  saveSettings: vi.fn().mockResolvedValue(undefined),
  setDefaultLayout: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../lib/stores", () => {
  const { writable } = require("svelte/store");
  const state = {
    monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
    active_layouts: [],
    saved_layouts: [],
    is_paused: false,
    settings: {
      default_gap: 4,
      default_margin: 8,
      accent_color: "#8b5cf6",
      language: "en",
      auto_start: false,
      first_run_completed: true,
      default_layout_id: null,
    },
  };
  return {
    currentState: writable(state),
    selectedMonitor: writable(null),
    activeLayout: writable(null),
    savedLayouts: writable([]),
    settings: writable(state.settings),
  };
});

vi.mock("../../lib/notifications", () => ({
  notify: vi.fn(),
  toastNotifications: { subscribe: vi.fn(() => vi.fn()) },
}));

const mockState = {
  monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
  active_layouts: [],
  saved_layouts: [],
  is_paused: false,
  settings: {
    default_gap: 4,
    default_margin: 8,
    accent_color: "#8b5cf6",
    language: "en",
    auto_start: false,
    first_run_completed: true,
    default_layout_id: null,
  },
};

describe("LayoutEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    cleanup();
    currentState.set(mockState);
    savedLayouts.set([]);
  });

  afterEach(() => {
    cleanup();
  });

  it("renders monitor name and resolution", async () => {
    const { findByText } = render(LayoutEditor);
    expect(await findByText("Main (1920×1080)")).toBeTruthy();
  });

  it("creates zone on canvas click", async () => {
    const { container, findByRole } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 400, clientY: 200 });
    expect(await findByRole("button", { name: /^Zone \d+$/ })).toBeTruthy();
  });

  it("shows confirmation on delete attempt", async () => {
    const { container, findByRole, findByText } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 200, clientY: 100 });
    const zone = await findByRole("button", { name: /^Zone \d+$/ });
    await fireEvent.click(zone);
    const deleteBtn = await findByText("Delete Zone");
    await fireEvent.click(deleteBtn);
    expect(await findByText(/Delete ".+"\?/)).toBeTruthy();
  });

  it("moves zone with arrow keys", async () => {
    const { container, findByRole } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 100, clientY: 100 });
    const zone = await findByRole("button", { name: /^Zone \d+$/ });
    (zone as HTMLElement).focus();
    await fireEvent.keyDown(zone, { key: "ArrowRight" });
  });

  it("does not call IPC on draft zone creation", async () => {
    const apply = vi.mocked(applyLayout);
    const save = vi.mocked(saveLayout);
    const { container } = render(LayoutEditor);
    const canvas = container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 200, clientY: 200 });
    expect(container.querySelector(".zone")).toBeTruthy();
    expect(apply).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
  });

  it("applies only on explicit action and preserves draft after failure", async () => {
    const apply = vi.mocked(applyLayout);
    apply.mockRejectedValueOnce(new Error("apply failed"));
    const view = render(LayoutEditor);
    const canvas = view.container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 120, clientY: 80 });
    expect(apply).not.toHaveBeenCalled();
    await fireEvent.click(view.getByRole("button", { name: "Apply Live" }));
    await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
    expect(view.getByRole("button", { name: /Zone 1/ })).toBeTruthy();
    expect(await view.findByText(/apply failed/i)).toBeTruthy();
    expect((view.getByRole("button", { name: "Apply Live" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("disables Apply Live while pending", async () => {
    const apply = vi.mocked(applyLayout);
    let resolveApply: (value: void) => void;
    apply.mockReturnValueOnce(new Promise<void>(resolve => { resolveApply = resolve; }));
    const view = render(LayoutEditor);
    const canvas = view.container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 120, clientY: 80 });
    await fireEvent.click(view.getByRole("button", { name: "Apply Live" }));
    await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
    expect((view.getByRole("button", { name: "Apply Live" }) as HTMLButtonElement).disabled).toBe(true);
    resolveApply!();
    await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
  });

  it("disables Save button while saving", async () => {
    const save = vi.mocked(saveLayout);
    let resolveSave: (value: void) => void;
    save.mockReturnValueOnce(new Promise<void>(resolve => { resolveSave = resolve; }));
    const view = render(LayoutEditor);
    const canvas = view.container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 120, clientY: 80 });
    const nameInput = view.container.querySelector(".save-input") as HTMLInputElement;
    await fireEvent.input(nameInput, { target: { value: "My Layout" } });
    expect((view.getByRole("button", { name: "Save" }) as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(view.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect((view.getByRole("button", { name: "Save" }) as HTMLButtonElement).disabled).toBe(true);
    resolveSave!();
    await waitFor(() => expect(nameInput.value).toBe(""));
  });

  it("preserves draft zones after apply rejection", async () => {
    const apply = vi.mocked(applyLayout);
    apply.mockRejectedValueOnce(new Error("apply failed"));
    const view = render(LayoutEditor);
    const canvas = view.container.querySelector(".monitor-canvas")!;
    await fireEvent.pointerDown(canvas, { clientX: 120, clientY: 80 });
    const zone = view.getByRole("button", { name: /Zone 1/ });
    expect(zone).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Apply Live" }));
    await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
    expect(view.getByRole("button", { name: /Zone 1/ })).toBeTruthy();
  });
});
