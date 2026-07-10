import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, waitFor, cleanup } from "@testing-library/svelte/svelte5";
import LayoutManager from "../LayoutManager.svelte";
import { listLayouts, setDefaultLayout, deleteLayout, getCurrentState } from "../../lib/ipc";
import { currentState, savedLayouts } from "../../lib/stores";
import type { SavedLayout, FrontendState } from "../../lib/types";

const layoutFixture: SavedLayout = {
  id: "layout-1", name: "My Layout", arrangement_id: "arr-1",
  zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 1, gap: 4, margin: 8 }],
  monitor_id: "m1",
};

const layoutFixture2: SavedLayout = {
  id: "layout-2", name: "Second Layout", arrangement_id: "arr-2",
  zones: [
    { id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 1, gap: 4, margin: 8 },
    { id: "z2", name: "Zone 2", x: 0.5, y: 0, width: 0.5, height: 1, gap: 4, margin: 8 },
  ],
  monitor_id: "m2",
};

const stateFixture: FrontendState = {
  monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
  active_layouts: [{ zones: [], monitor_id: "m1" }],
  saved_layouts: [layoutFixture],
  is_paused: false,
  settings: { auto_start: false, default_gap: 4, default_margin: 8, accent_color: "#8B5CF6", language: "en", first_run_completed: true, default_layout_id: null },
};

vi.mock("../../lib/ipc", () => ({
  listLayouts: vi.fn(),
  setDefaultLayout: vi.fn(),
  deleteLayout: vi.fn(),
  getCurrentState: vi.fn(),
}));

vi.mock("../../lib/stores", () => {
  const { writable } = require("svelte/store");
  const defaultState = {
    monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
    active_layouts: [{ zones: [], monitor_id: "m1" }],
    saved_layouts: [{
      id: "layout-1", name: "My Layout", arrangement_id: "arr-1",
      zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 1, gap: 4, margin: 8 }],
      monitor_id: "m1",
    }],
    is_paused: false,
    settings: { auto_start: false, default_gap: 4, default_margin: 8, accent_color: "#8B5CF6", language: "en", first_run_completed: true, default_layout_id: null },
  };
  return {
    currentState: writable(defaultState),
    selectedMonitor: writable(null),
    activeLayout: writable(null),
    savedLayouts: writable(defaultState.saved_layouts),
    settings: writable(defaultState.settings),
  };
});

vi.mock("../../lib/notifications", () => ({
  notify: vi.fn(),
  toastNotifications: { subscribe: vi.fn(() => vi.fn()) },
}));

import { notify } from "../../lib/notifications";

describe("LayoutManager", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    cleanup();
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([layoutFixture]));
    vi.mocked(setDefaultLayout).mockImplementation(() => Promise.resolve());
    vi.mocked(deleteLayout).mockImplementation(() => Promise.resolve());
    vi.mocked(getCurrentState).mockImplementation(() => Promise.resolve(stateFixture));
    currentState.set(stateFixture);
    savedLayouts.set([layoutFixture]);
  });

  afterEach(() => {
    cleanup();
  });

  it("renders layout name and zone count", async () => {
    const view = render(LayoutManager);
    expect(await view.findByText("1 zone")).toBeTruthy();
    const matches = view.getAllByText("My Layout");
    expect(matches.length).toBeGreaterThanOrEqual(1);
  });

  it("shows empty state when no layouts", async () => {
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([]));
    const view = render(LayoutManager);
    expect(await view.findByText("No layouts saved")).toBeTruthy();
  });

  it("shows DEFAULT badge when settings.default_layout_id matches", async () => {
    currentState.set({ ...stateFixture, settings: { ...stateFixture.settings, default_layout_id: layoutFixture.id } });
    const view = render(LayoutManager);
    expect(await view.findByText("Default")).toBeTruthy();
  });

  it("shows ACTIVE badge when active_layouts monitor_id matches", async () => {
    currentState.set({ ...stateFixture, active_layouts: [{ zones: [], monitor_id: "m1" }] });
    const view = render(LayoutManager);
    expect(await view.findByText("Active")).toBeTruthy();
  });

  it("refreshes default badge after setting a default", async () => {
    vi.mocked(setDefaultLayout).mockImplementation(() => Promise.resolve());
    vi.mocked(getCurrentState).mockImplementation(() => Promise.resolve({ ...stateFixture, settings: { ...stateFixture.settings, default_layout_id: layoutFixture.id } }));
    const view = render(LayoutManager);
    await fireEvent.click(await view.findByRole("button", { name: "Set Default" }));
    await waitFor(() => expect(setDefaultLayout).toHaveBeenCalledWith(layoutFixture.id));
    expect(await view.findByText("Default")).toBeTruthy();
  });

  it("removes layout after successful delete", async () => {
    window.confirm = vi.fn(() => true);
    vi.mocked(listLayouts)
      .mockImplementationOnce(() => Promise.resolve([layoutFixture]))
      .mockImplementationOnce(() => Promise.resolve([]));
    vi.mocked(deleteLayout).mockImplementation(() => Promise.resolve());
    const view = render(LayoutManager);
    const btn = await view.findByRole("button", { name: "Delete" });
    expect(view.getAllByText("My Layout").length).toBeGreaterThanOrEqual(1);
    await fireEvent.click(btn);
    await waitFor(() => expect(deleteLayout).toHaveBeenCalledWith(layoutFixture.id));
    await waitFor(() => {
      expect(view.queryByText("My Layout")).toBeNull();
    });
  });

  it("disables Set Default button while pending", async () => {
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([layoutFixture]));
    let resolveOp: (value: void) => void;
    vi.mocked(setDefaultLayout).mockImplementationOnce(() => new Promise<void>(resolve => { resolveOp = resolve; }));
    const view = render(LayoutManager);
    const btn = await view.findByRole("button", { name: "Set Default" });
    await fireEvent.click(btn);
    await waitFor(() => expect(setDefaultLayout).toHaveBeenCalledTimes(1));
    expect((btn as HTMLButtonElement).disabled).toBe(true);
    resolveOp!();
    await waitFor(() => expect((btn as HTMLButtonElement).disabled).toBe(false));
  });

  it("suppresses duplicate Set Default clicks", async () => {
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([layoutFixture]));
    let resolveOp: (value: void) => void;
    vi.mocked(setDefaultLayout).mockImplementationOnce(() => new Promise<void>(resolve => { resolveOp = resolve; }));
    const view = render(LayoutManager);
    const btn = await view.findByRole("button", { name: "Set Default" });
    await fireEvent.click(btn);
    await fireEvent.click(btn);
    expect(setDefaultLayout).toHaveBeenCalledTimes(1);
    resolveOp!();
  });

  it("re-enables button after setDefaultLayout rejects", async () => {
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([layoutFixture]));
    vi.mocked(setDefaultLayout).mockImplementationOnce(() => Promise.reject(new Error("failed")));
    const view = render(LayoutManager);
    const btn = await view.findByRole("button", { name: "Set Default" });
    await fireEvent.click(btn);
    await waitFor(() => expect(setDefaultLayout).toHaveBeenCalledTimes(1));
    expect((btn as HTMLButtonElement).disabled).toBe(false);
  });

  it("shows error notification on setDefaultLayout failure", async () => {
    vi.mocked(setDefaultLayout).mockImplementationOnce(() => Promise.reject(new Error("failed")));
    const view = render(LayoutManager);
    await fireEvent.click(await view.findByRole("button", { name: "Set Default" }));
    await waitFor(() => expect(notify).toHaveBeenCalledWith(expect.stringContaining("Failed to set default"), "error"));
  });

  it("shows error notification on deleteLayout failure", async () => {
    window.confirm = vi.fn(() => true);
    vi.mocked(deleteLayout).mockImplementationOnce(() => Promise.reject(new Error("delete failed")));
    const view = render(LayoutManager);
    await fireEvent.click(await view.findByRole("button", { name: "Delete" }));
    await waitFor(() => expect(notify).toHaveBeenCalledWith(expect.stringContaining("Failed to delete"), "error"));
  });

  it("renders multiple layouts with correct metadata", async () => {
    vi.mocked(listLayouts).mockImplementation(() => Promise.resolve([layoutFixture, layoutFixture2]));
    const view = render(LayoutManager);
    expect(await view.findByText("1 zone")).toBeTruthy();
    expect(await view.findByText("2 zones")).toBeTruthy();
    expect(view.getAllByText("My Layout").length).toBeGreaterThanOrEqual(1);
    expect(view.getAllByText("Second Layout").length).toBeGreaterThanOrEqual(1);
  });

  it("refreshes stores after successful delete", async () => {
    window.confirm = vi.fn(() => true);
    vi.mocked(deleteLayout).mockImplementation(() => Promise.resolve());
    const freshState = { ...stateFixture, saved_layouts: [] };
    vi.mocked(getCurrentState).mockImplementation(() => Promise.resolve(freshState));
    vi.mocked(listLayouts)
      .mockImplementationOnce(() => Promise.resolve([layoutFixture]))
      .mockImplementationOnce(() => Promise.resolve([]));
    const view = render(LayoutManager);
    await fireEvent.click(await view.findByRole("button", { name: "Delete" }));
    await waitFor(() => expect(getCurrentState).toHaveBeenCalled());
  });
});
