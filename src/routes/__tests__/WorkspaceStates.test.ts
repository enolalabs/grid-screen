import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte/svelte5";
import AppShell from "../../lib/components/AppShell.svelte";

function createState(overrides: Record<string, unknown> = {}) {
  return {
    monitors: [
      { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ],
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
    ...overrides,
  };
}

describe("WorkspaceStates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    cleanup();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders onboarding empty state when first_run_completed is false", async () => {
    const state = createState({
      settings: { ...createState().settings, first_run_completed: false },
    });
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    } as never);
    expect(await findByText("Welcome to Grid Screen")).toBeTruthy();
  });

  it("shows Open Workspace action in onboarding state", async () => {
    const state = createState({
      settings: { ...createState().settings, first_run_completed: false },
    });
    const onNavigate = vi.fn();
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate,
        onRetry: vi.fn(),
      },
    } as never);
    const btn = await findByText("Open Workspace");
    await fireEvent.click(btn);
    expect(onNavigate).toHaveBeenCalledWith("workspace");
  });

  it("calls onCompleteOnboarding on Finish setup click", async () => {
    const state = createState({
      settings: { ...createState().settings, first_run_completed: false },
    });
    const onCompleteOnboarding = vi.fn();
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
        onCompleteOnboarding,
      },
    } as never);
    const btn = await findByText("Finish setup");
    await fireEvent.click(btn);
    expect(onCompleteOnboarding).toHaveBeenCalledTimes(1);
  });

  it("renders empty state when no layouts saved", async () => {
    const state = createState({
      saved_layouts: [],
      settings: { ...createState().settings, first_run_completed: true },
    });
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    } as never);
    expect(await findByText("No layouts saved")).toBeTruthy();
  });

  it("navigates to workspace from empty state", async () => {
    const state = createState({
      saved_layouts: [],
      settings: { ...createState().settings, first_run_completed: true },
    });
    const onNavigate = vi.fn();
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate,
        onRetry: vi.fn(),
      },
    } as never);
    const btn = await findByText("Open Workspace");
    await fireEvent.click(btn);
    expect(onNavigate).toHaveBeenCalledWith("workspace");
  });

  it("renders recovery state when saved layouts exist but no monitors", async () => {
    const state = createState({
      monitors: [],
      saved_layouts: [
        {
          id: "l1",
          name: "My Layout",
          arrangement_id: "m1",
          zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 0.5, gap: 4, margin: 8 }],
          monitor_id: "m1",
        },
      ],
      settings: { ...createState().settings, first_run_completed: true },
    });
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 0,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    } as never);
    expect(await findByText("No monitors detected")).toBeTruthy();
  });

  it("navigates to status from recovery state", async () => {
    const state = createState({
      monitors: [],
      saved_layouts: [
        {
          id: "l1",
          name: "My Layout",
          arrangement_id: "m1",
          zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 0.5, gap: 4, margin: 8 }],
          monitor_id: "m1",
        },
      ],
      settings: { ...createState().settings, first_run_completed: true },
    });
    const onNavigate = vi.fn();
    const { findByText } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 0,
        onNavigate,
        onRetry: vi.fn(),
      },
    } as never);
    const btn = await findByText("Check System Status");
    await fireEvent.click(btn);
    expect(onNavigate).toHaveBeenCalledWith("status");
  });

  it("renders children in ready state", async () => {
    const state = createState({
      saved_layouts: [
        {
          id: "l1",
          name: "My Layout",
          arrangement_id: "m1",
          zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 0.5, gap: 4, margin: 8 }],
          monitor_id: "m1",
        },
      ],
      settings: { ...createState().settings, first_run_completed: true },
    });
    const { container } = render(AppShell, {
      props: {
        activeView: "workspace",
        initialization: { status: "loaded", state },
        isPaused: false,
        monitorCount: 1,
        onNavigate: vi.fn(),
        onRetry: vi.fn(),
      },
    } as never);
    expect(container.querySelector(".app-content")).toBeTruthy();
  });
});
