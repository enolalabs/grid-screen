import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, waitFor, cleanup } from "@testing-library/svelte/svelte5";
import Settings from "../Settings.svelte";
import { getSettings, saveSettings, getCurrentState } from "../../lib/ipc";
import { currentState, savedLayouts, settings as settingsStore } from "../../lib/stores";
import type { AppSettings, FrontendState } from "../../lib/types";

const settingsFixture: AppSettings = {
  auto_start: false,
  default_gap: 4,
  default_margin: 8,
  accent_color: "#8B5CF6",
  language: "en",
  first_run_completed: true,
  default_layout_id: "layout-1",
};

const stateFixture: FrontendState = {
  monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
  active_layouts: [],
  saved_layouts: [],
  is_paused: false,
  settings: settingsFixture,
};

vi.mock("../../lib/ipc", () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  getCurrentState: vi.fn(),
}));

vi.mock("../../lib/stores", () => {
  const { writable } = require("svelte/store");
  return {
    currentState: writable<FrontendState | null>(null),
    savedLayouts: writable([]),
    settings: writable<AppSettings | null>(null),
  };
});

vi.mock("../../lib/notifications", () => ({
  notify: vi.fn(),
  toastNotifications: { subscribe: vi.fn(() => vi.fn()) },
  notificationHistory: { subscribe: vi.fn(() => vi.fn()) },
  clearNotificationHistory: vi.fn(),
}));

import { notify } from "../../lib/notifications";

describe("Settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    cleanup();
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockResolvedValue();
    vi.mocked(getCurrentState).mockResolvedValue(stateFixture);
    currentState.set(stateFixture);
    savedLayouts.set([]);
    settingsStore.set(settingsFixture);
  });

  afterEach(() => {
    cleanup();
  });

  it("loads settings from getSettings on mount", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    render(Settings);
    await waitFor(() => expect(getSettings).toHaveBeenCalled());
  });

  it("renders all settings fields", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    const view = render(Settings);
    expect(await view.findByLabelText("Auto-start with system")).toBeTruthy();
    expect(await view.findByLabelText("Default gap between zones (px)")).toBeTruthy();
    expect(await view.findByLabelText("Default margin from screen edge (px)")).toBeTruthy();
    expect(await view.findByLabelText("Accent color")).toBeTruthy();
    expect(await view.findByLabelText("Language")).toBeTruthy();
    expect(await view.findByText("About Grid Screen")).toBeTruthy();
  });

  it("renders default_layout_id field", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    const view = render(Settings);
    expect(await view.findByLabelText("Default layout ID")).toBeTruthy();
  });

  it("pre-populates fields with loaded settings", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    const view = render(Settings);
    const gap = (await view.findByLabelText("Default gap between zones (px)")) as HTMLInputElement;
    expect(gap.value).toBe("4");
    const margin = (await view.findByLabelText("Default margin from screen edge (px)")) as HTMLInputElement;
    expect(margin.value).toBe("8");
  });

  it("save success: calls saveSettings then getCurrentState and updates stores", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockResolvedValue();
    vi.mocked(getCurrentState).mockResolvedValue(stateFixture);
    const view = render(Settings);
    const btn = await view.findByRole("button", { name: "Save Settings" });
    await fireEvent.click(btn);
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ default_gap: 4 })));
    await waitFor(() => expect(getCurrentState).toHaveBeenCalled());
  });

  it("save failure: shows ErrorPanel with message", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));
    const view = render(Settings);
    await fireEvent.click(view.getByRole("button", { name: "Save Settings" }));
    expect(await view.findByText("Save Failed")).toBeTruthy();
  });

  it("retains edited settings and restores Save after failure", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));
    const view = render(Settings);
    await waitFor(() => expect(getSettings).toHaveBeenCalled());
    const gap = (await view.findByLabelText("Default gap between zones (px)")) as HTMLInputElement;
    gap.value = "12";
    gap.dispatchEvent(new Event("input", { bubbles: true }));
    await fireEvent.click(view.getByRole("button", { name: "Save Settings" }));
    expect(await view.findByText("Save Failed")).toBeTruthy();
    expect((gap as HTMLInputElement).value).toBe("12");
    expect((view.getByRole("button", { name: "Save Settings" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("disables Save button during pending", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    let resolveOp: (value: void) => void;
    vi.mocked(saveSettings).mockImplementationOnce(() => new Promise<void>(resolve => { resolveOp = resolve; }));
    const view = render(Settings);
    const btn = await view.findByRole("button", { name: "Save Settings" });
    await fireEvent.click(btn);
    await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
    expect((btn as HTMLButtonElement).disabled).toBe(true);
    resolveOp!();
    await waitFor(() => expect((btn as HTMLButtonElement).disabled).toBe(false));
  });

  it("suppresses duplicate save clicks (2 quick clicks → 1 IPC call)", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    let resolveOp: (value: void) => void;
    vi.mocked(saveSettings).mockImplementationOnce(() => new Promise<void>(resolve => { resolveOp = resolve; }));
    const view = render(Settings);
    const btn = await view.findByRole("button", { name: "Save Settings" });
    await fireEvent.click(btn);
    await fireEvent.click(btn);
    expect(saveSettings).toHaveBeenCalledTimes(1);
    resolveOp!();
  });

  it("re-enables Save button after saveSettings rejects", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));
    const view = render(Settings);
    const btn = await view.findByRole("button", { name: "Save Settings" });
    await fireEvent.click(btn);
    await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
    expect((btn as HTMLButtonElement).disabled).toBe(false);
  });

  it("sends notification toast on save failure", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));
    const view = render(Settings);
    await fireEvent.click(view.getByRole("button", { name: "Save Settings" }));
    await waitFor(() => expect(notify).toHaveBeenCalledWith(expect.stringContaining("Failed to save"), "error"));
  });

  it("shows success feedback after save", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    vi.mocked(saveSettings).mockResolvedValue();
    const view = render(Settings);
    await fireEvent.click(view.getByRole("button", { name: "Save Settings" }));
    expect(await view.findByRole("button", { name: "Saved!" })).toBeTruthy();
  });

  it("renders About panel with version info", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    const view = render(Settings);
    expect(await view.findByText("Version 0.1.0")).toBeTruthy();
    expect(await view.findByText("Cross-platform window zone management.")).toBeTruthy();
  });

  it("renders language select with options", async () => {
    vi.mocked(getSettings).mockResolvedValue(settingsFixture);
    const view = render(Settings);
    const select = await view.findByLabelText("Language") as HTMLSelectElement;
    expect(select.value).toBe("en");
    const options = Array.from(select.options).map(o => o.value);
    expect(options).toContain("en");
    expect(options).toContain("vi");
  });

  it("auto_start checkbox reflects loaded value", async () => {
    vi.mocked(getSettings).mockResolvedValue({ ...settingsFixture, auto_start: true });
    const view = render(Settings);
    const cb = (await view.findByLabelText("Auto-start with system")) as HTMLInputElement;
    await waitFor(() => expect(cb.checked).toBe(true));
  });
});
