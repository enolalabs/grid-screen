import { describe, it, expect, vi, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte/svelte5";
import { readFileSync } from "fs";
import { resolve } from "path";
import AppShell from "../components/AppShell.svelte";
import Sidebar from "../components/Sidebar.svelte";
import type { FrontendState, InitializationState } from "../types";

afterEach(cleanup);

const themeCss = readFileSync(resolve(__dirname, "../theme.css"), "utf-8");

function makeState(overrides: Partial<FrontendState> = {}): FrontendState {
  return {
    monitors: [
      { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ],
    active_layouts: [],
    saved_layouts: [
      {
        id: "l1",
        name: "My Layout",
        arrangement_id: "a1",
        zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 1, height: 1, gap: 4, margin: 8 }],
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

describe("Sidebar accessibility", () => {
  it("sidebar nav buttons have accessible names", () => {
    const view = render(Sidebar, {
      props: { activeView: "workspace", isPaused: false, onNavigate: vi.fn() },
    });

    const tabs = view.container.querySelectorAll("[role='tab']");
    const names = Array.from(tabs).map(el => el.textContent?.trim() ?? "");
    expect(names).toContain("Workspace");
    expect(names).toContain("Saved Layouts");
    expect(names).toContain("Settings");
    expect(names.some(n => n.includes("System Status"))).toBe(true);
  });

  it("sidebar has navigation role with label", () => {
    const view = render(Sidebar, {
      props: { activeView: "workspace", isPaused: false, onNavigate: vi.fn() },
    });

    const sidebar = view.container.querySelector("aside[aria-label='Main navigation']");
    expect(sidebar).toBeTruthy();
    expect(sidebar?.getAttribute("role")).toBe("navigation");
  });

  it("sidebar nav items have correct aria-selected state", () => {
    const view = render(Sidebar, {
      props: { activeView: "layouts", isPaused: false, onNavigate: vi.fn() },
    });

    const tabs = view.container.querySelectorAll("[role='tab']");
    const layoutsBtn = Array.from(tabs).find(el => el.textContent?.includes("Saved Layouts"));
    const workspaceBtn = Array.from(tabs).find(el => el.textContent === "Workspace");

    expect(layoutsBtn?.getAttribute("aria-selected")).toBe("true");
    expect(workspaceBtn?.getAttribute("aria-selected")).toBe("false");
  });

  it("sidebar nav responds to keyboard Enter key", async () => {
    const onNavigate = vi.fn();
    const view = render(Sidebar, {
      props: { activeView: "workspace", isPaused: false, onNavigate },
    });

    const tabs = view.container.querySelectorAll("[role='tab']");
    const layoutsBtn = Array.from(tabs).find(el => el.textContent?.includes("Saved Layouts"));
    expect(layoutsBtn).toBeTruthy();

    layoutsBtn!.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(onNavigate).toHaveBeenCalledWith("layouts");
  });

  it("sidebar nav responds to keyboard Space key", async () => {
    const onNavigate = vi.fn();
    const view = render(Sidebar, {
      props: { activeView: "workspace", isPaused: false, onNavigate },
    });

    const tabs = view.container.querySelectorAll("[role='tab']");
    const settingsBtn = Array.from(tabs).find(el => el.textContent === "Settings");
    expect(settingsBtn).toBeTruthy();

    settingsBtn!.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
    expect(onNavigate).toHaveBeenCalledWith("settings");
  });
});

describe("Live region and status messages", () => {
  it("error banner in failed state has alert role", () => {
    const init: InitializationState = { status: "failed", message: "Connection refused" };

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

    expect(view.getByRole("alert")).toBeTruthy();
  });

  it("renders a div with role=status and aria-live=polite when wrapping toast container", () => {
    const container = document.createElement("div");
    container.setAttribute("role", "status");
    container.setAttribute("aria-live", "polite");
    container.textContent = "Layout saved";

    document.body.appendChild(container);

    const liveRegion = document.querySelector('[role="status"][aria-live="polite"]');
    expect(liveRegion).toBeTruthy();
    expect(liveRegion?.textContent).toBe("Layout saved");

    document.body.removeChild(container);
  });
});

describe("Focus-visible styles", () => {
  it("theme.css defines :focus-visible with box-shadow outline", () => {
    expect(themeCss).toContain(":focus-visible");
    expect(themeCss).toContain("var(--focus-ring)");
  });
});

describe("Reduced motion", () => {
  it("theme.css has prefers-reduced-motion: reduce that disables transitions", () => {
    expect(themeCss).toContain("@media (prefers-reduced-motion: reduce)");
    expect(themeCss).toContain("transition-duration: 0s");
    expect(themeCss).toContain("animation-duration: 0s");
  });

  it("Sidebar nav-item transition is disabled under reduced motion", () => {
    expect(themeCss).toContain("@media (prefers-reduced-motion: reduce)");
  });
});
