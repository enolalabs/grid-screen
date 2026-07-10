import { describe, it, expect, vi, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte/svelte5";
import type { InitializationState, FrontendState } from "../../types";
import AppShell from "../AppShell.svelte";

afterEach(cleanup);

function makeState(overrides: Partial<FrontendState> & { settings?: Partial<FrontendState["settings"]> } = {}): FrontendState {
  const { settings: settingsOverrides, ...topOverrides } = overrides;
  return {
    monitors: [],
    active_layouts: [],
    saved_layouts: [],
    is_paused: false,
    settings: {
      auto_start: false,
      default_gap: 4,
      default_margin: 8,
      accent_color: "#8B5CF6",
      language: "en",
      first_run_completed: false,
      default_layout_id: null,
      ...settingsOverrides,
    },
    ...topOverrides,
  };
}

describe("AppShell", () => {
  it("renders loading state", () => {
    const init: InitializationState = { status: "loading" };
    const view = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: init,
        isPaused: false,
        monitorCount: 0,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    });

    expect(view.getByText("INITIALIZING")).toBeTruthy();
    expect(view.getByText("Loading")).toBeTruthy();
    expect(view.getByText("Connecting to the Grid Screen service...")).toBeTruthy();
  });

  it("renders error state with retry", async () => {
    const onRetry = vi.fn();
    const init: InitializationState = { status: "failed", message: "Connection refused" };
    const view = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: init,
        isPaused: false,
        monitorCount: 0,
        onNavigate: vi.fn(),
        onRetry,
      },
    });

    expect(view.getByText("Connection Failed")).toBeTruthy();
    expect(view.getByText("Connection refused")).toBeTruthy();

    await fireEvent.click(view.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("renders onboarding state for first run", () => {
    const state = makeState();
    const init: InitializationState = { status: "loaded", state };

    const view = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: init,
        isPaused: false,
        monitorCount: 0,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    });

    expect(view.getByText("FIRST RUN")).toBeTruthy();
    expect(view.getByText("Welcome to Grid Screen")).toBeTruthy();
  });

  it("fires onNavigate when sidebar nav items are clicked", async () => {
    const state = makeState({
      monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
      saved_layouts: [{ id: "l1", name: "My Layout", arrangement_id: "a1", zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }], monitor_id: "m1" }],
      settings: { first_run_completed: true },
    });
    const init: InitializationState = { status: "loaded", state };
    const onNavigate = vi.fn();

    const view = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: init,
        isPaused: false,
        monitorCount: 1,
        onNavigate,
        onRetry: vi.fn(),
      },
    });

    const layoutsBtn = view.getByRole("tab", { name: "Saved Layouts" });
    await fireEvent.click(layoutsBtn);
    expect(onNavigate).toHaveBeenCalledWith("layouts");
  });

  it("renders slot content in ready state", () => {
    const state = makeState({
      monitors: [{ id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true }],
      saved_layouts: [{ id: "l1", name: "My Layout", arrangement_id: "a1", zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }], monitor_id: "m1" }],
      settings: { first_run_completed: true },
    });
    const init: InitializationState = { status: "loaded", state };

    const view = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: init,
        isPaused: false,
        monitorCount: 1,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
        children: "Route Content",
      },
    });

    expect(view.getByText("Route Content")).toBeTruthy();
  });
});
