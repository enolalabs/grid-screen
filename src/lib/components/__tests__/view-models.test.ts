import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import type { FrontendState } from "../../types";
import {
  isUsableLayout,
  getFirstRunState,
  isDefaultLayout,
  isActiveLayout,
} from "../../view-models";
import { notify, notificationHistory, toastNotifications, clearNotificationHistory } from "../../notifications";

describe("view-models", () => {
  it("getFirstRunState returns onboarding when first_run_completed is false", () => {
    const state: FrontendState = {
      monitors: [
        { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
      ],
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
      },
    };

    expect(getFirstRunState(state)).toBe("onboarding");
  });

  it("isUsableLayout returns false when layout has no zones", () => {
    const monitors = [
      { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ];

    expect(isUsableLayout({ zones: [], monitor_id: "m1" }, monitors)).toBe(false);
  });

  it("isUsableLayout returns false when monitor does not exist", () => {
    const monitors = [
      { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ];

    expect(
      isUsableLayout({ zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 0.5, gap: 4, margin: 8 }], monitor_id: "m2" }, monitors),
    ).toBe(false);
  });

  it("isUsableLayout returns true when layout has zones and monitor exists", () => {
    const monitors = [
      { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
    ];

    expect(
      isUsableLayout({ zones: [{ id: "z1", name: "Zone 1", x: 0, y: 0, width: 0.5, height: 0.5, gap: 4, margin: 8 }], monitor_id: "m1" }, monitors),
    ).toBe(true);
  });

  it("getFirstRunState returns empty when first_run is complete but no saved layouts", () => {
    const state: FrontendState = {
      monitors: [
        { id: "m1", name: "Main", x: 0, y: 0, width: 1920, height: 1080, dpi_scale: 1, is_primary: true },
      ],
      active_layouts: [],
      saved_layouts: [],
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
    };

    expect(getFirstRunState(state)).toBe("empty");
  });

  it("getFirstRunState returns recovery when monitors are empty", () => {
    const state: FrontendState = {
      monitors: [],
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
    };

    expect(getFirstRunState(state)).toBe("recovery");
  });

  it("getFirstRunState returns ready when everything is set up", () => {
    const state: FrontendState = {
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
    };

    expect(getFirstRunState(state)).toBe("ready");
  });

  it("isDefaultLayout checks settings.default_layout_id", () => {
    const layout = { id: "l1" };
    const settings = {
      auto_start: false,
      default_gap: 4,
      default_margin: 8,
      accent_color: "#8B5CF6",
      language: "en",
      first_run_completed: false,
      default_layout_id: "l1",
    };

    expect(isDefaultLayout(layout, settings)).toBe(true);
    expect(isDefaultLayout({ id: "l2" }, settings)).toBe(false);
  });

  it("isActiveLayout checks active_layouts monitor IDs", () => {
    const layout = { monitor_id: "m1" };
    const activeLayouts = [
      { zones: [], monitor_id: "m1" },
      { zones: [], monitor_id: "m2" },
    ];

    expect(isActiveLayout(layout, activeLayouts)).toBe(true);
    expect(isActiveLayout({ monitor_id: "m3" }, activeLayouts)).toBe(false);
    expect(isActiveLayout({ monitor_id: "m1" }, [])).toBe(false);
  });
});

describe("notifications", () => {
  beforeEach(() => {
    toastNotifications.set([]);
    clearNotificationHistory();
  });

  it("notificationHistory caps at 100 entries", () => {
    for (let index = 0; index < 101; index += 1) notify(`message-${index}`, "info");
    expect(get(notificationHistory)).toHaveLength(100);
  });

  it("notificationHistory coalesces consecutive identical message/type entries", () => {
    notify("duplicate", "info");
    notify("duplicate", "info");
    expect(get(notificationHistory)).toHaveLength(1);
  });

  it("notificationHistory does not coalesce different messages", () => {
    notify("msg-1", "info");
    notify("msg-2", "info");
    expect(get(notificationHistory)).toHaveLength(2);
  });

  it("notificationHistory does not coalesce same message with different type", () => {
    notify("msg", "info");
    notify("msg", "error");
    expect(get(notificationHistory)).toHaveLength(2);
  });

  it("clearNotificationHistory empties the history", () => {
    notify("msg", "info");
    clearNotificationHistory();
    expect(get(notificationHistory)).toEqual([]);
  });

  it("toastNotifications auto-expire entries", { timeout: 10000 }, async () => {
    notify("transient", "info");
    expect(get(toastNotifications)).toHaveLength(1);

    await new Promise(resolve => setTimeout(resolve, 5100));
    expect(get(toastNotifications)).toHaveLength(0);
  });
});
