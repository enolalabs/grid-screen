import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte/svelte5";
import SystemStatus from "../SystemStatus.svelte";
import type { FrontendState, Notification } from "../../lib/types";

const settingsFixture = {
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
  saved_layouts: [
    {
      id: "layout-1",
      name: "My Layout",
      arrangement_id: "arr-1",
      zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 1, gap: 4, margin: 8 }],
      monitor_id: "m1",
    },
  ],
  is_paused: false,
  settings: settingsFixture,
};

const historyFixture: Notification[] = [
  { id: "n1", message: "Layout applied", type: "info" },
  { id: "n2", message: "Connection lost", type: "error" },
];

describe("SystemStatus", () => {
  let onRetry: ReturnType<typeof vi.fn>;
  let onClearHistory: ReturnType<typeof vi.fn>;
  let onNavigate: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    cleanup();
    onRetry = vi.fn();
    onClearHistory = vi.fn();
    onNavigate = vi.fn();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders monitor count", () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Monitors")).toBeTruthy();
    expect(view.getAllByText("1").length).toBeGreaterThanOrEqual(1);
  });

  it("renders saved layouts count", () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Saved layouts")).toBeTruthy();
  });

  it("shows paused warning when isPaused", () => {
    const pausedState = { ...stateFixture, is_paused: true };
    const view = render(SystemStatus, {
      state: pausedState,
      initialization: { status: "loaded", state: pausedState },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Yes")).toBeTruthy();
  });

  it("renders notification history entries", () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: historyFixture,
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Layout applied")).toBeTruthy();
    expect(view.getByText("Connection lost")).toBeTruthy();
  });

  it("Clear button calls onClearHistory", async () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: historyFixture,
      onRetry,
      onClearHistory,
      onNavigate,
    });
    const btn = view.getByRole("button", { name: "Clear" });
    await fireEvent.click(btn);
    expect(onClearHistory).toHaveBeenCalledTimes(1);
  });

  it("shows loading state when initialization is loading", () => {
    const view = render(SystemStatus, {
      state: null,
      initialization: { status: "loading" },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Loading")).toBeTruthy();
  });

  it("shows ErrorPanel when initialization failed", () => {
    const view = render(SystemStatus, {
      state: null,
      initialization: { status: "failed", message: "Connection error" },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("Connection Failed")).toBeTruthy();
    expect(view.getByText("Connection error")).toBeTruthy();
  });

  it("ErrorPanel retry button calls onRetry", async () => {
    const view = render(SystemStatus, {
      state: null,
      initialization: { status: "failed", message: "Connection error" },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    const retryBtn = view.getByRole("button", { name: "Retry" });
    await fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("Back to Workspace button calls onNavigate", async () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    const btn = view.getByRole("button", { name: "Back to Workspace" });
    await fireEvent.click(btn);
    expect(onNavigate).toHaveBeenCalledWith("workspace");
  });

  it("shows empty history message when no notifications", () => {
    const view = render(SystemStatus, {
      state: stateFixture,
      initialization: { status: "loaded", state: stateFixture },
      history: [],
      onRetry,
      onClearHistory,
      onNavigate,
    });
    expect(view.getByText("No recent notifications")).toBeTruthy();
  });
});
