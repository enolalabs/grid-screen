import { describe, it, expect, vi, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte/svelte5";

afterEach(cleanup);
import Button from "../Button.svelte";
import Badge from "../Badge.svelte";
import Panel from "../Panel.svelte";
import StatusRow from "../StatusRow.svelte";
import EmptyState from "../EmptyState.svelte";
import ErrorPanel from "../ErrorPanel.svelte";

describe("Button", () => {
  it("renders with accessible name from slot text", () => {
    const view = render(Button, { props: { children: "Save" } });
    expect(view.getByRole("button", { name: "Save" })).toBeTruthy();
  });

  it("renders with ariaLabel as accessible name", () => {
    const view = render(Button, { props: { ariaLabel: "Delete item", children: "X" } });
    expect(view.getByRole("button", { name: "Delete item" })).toBeTruthy();
  });

  it("applies primary variant class by default", () => {
    const view = render(Button, { props: { children: "Click" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.className).toContain("btn-primary");
  });

  it("applies ghost variant class", () => {
    const view = render(Button, { props: { variant: "ghost", children: "Cancel" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.className).toContain("btn-ghost");
  });

  it("applies danger variant class", () => {
    const view = render(Button, { props: { variant: "danger", children: "Delete" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.className).toContain("btn-danger");
  });

  it("sets button type attribute", () => {
    const view = render(Button, { props: { type: "submit", children: "Submit" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.type).toBe("submit");
  });

  it("sets default button type to button", () => {
    const view = render(Button, { props: { children: "Click" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.type).toBe("button");
  });

  it("sets disabled attribute when disabled", () => {
    const view = render(Button, { props: { disabled: true, children: "Nope" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("sets disabled and aria-busy when loading", () => {
    const view = render(Button, { props: { loading: true, children: "Wait" } });
    const btn = view.getByRole("button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
    expect(btn.getAttribute("aria-busy")).toBe("true");
  });

  it("fires click handler", async () => {
    const onClick = vi.fn();
    const view = render(Button, { props: { onclick: onClick, children: "Go" } });
    await fireEvent.click(view.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });
});

describe("Badge", () => {
  it("renders text content", () => {
    const view = render(Badge, { props: { tone: "primary", text: "Active" } });
    expect(view.getByText("Active")).toBeTruthy();
  });

  it("applies primary tone class", () => {
    const view = render(Badge, { props: { tone: "primary", text: "P" } });
    expect(view.container.querySelector(".badge-primary")).toBeTruthy();
  });

  it("applies success tone class", () => {
    const view = render(Badge, { props: { tone: "success", text: "OK" } });
    expect(view.container.querySelector(".badge-success")).toBeTruthy();
  });

  it("applies warning tone class", () => {
    const view = render(Badge, { props: { tone: "warning", text: "Warn" } });
    expect(view.container.querySelector(".badge-warning")).toBeTruthy();
  });

  it("applies error tone class", () => {
    const view = render(Badge, { props: { tone: "error", text: "Err" } });
    expect(view.container.querySelector(".badge-error")).toBeTruthy();
  });

  it("applies muted tone class", () => {
    const view = render(Badge, { props: { tone: "muted", text: "Off" } });
    expect(view.container.querySelector(".badge-muted")).toBeTruthy();
  });

  it("has status role for accessibility", () => {
    const view = render(Badge, { props: { tone: "muted", text: "Status" } });
    expect(view.getByRole("status")).toBeTruthy();
  });
});

describe("Panel", () => {
  it("renders title and eyebrow", () => {
    const view = render(Panel, { props: { title: "Monitor", eyebrow: "Devices" } });
    expect(view.getByText("Monitor")).toBeTruthy();
    expect(view.getByText("Devices")).toBeTruthy();
  });

  it("renders default slot content", () => {
    const view = render(Panel, { props: { title: "Panel", children: "Body content" } });
    expect(view.getByText("Body content")).toBeTruthy();
  });

  it("renders without header when no title or eyebrow", () => {
    const view = render(Panel, { props: { children: "Only body" } });
    expect(view.getByText("Only body")).toBeTruthy();
    expect(view.container.querySelector(".panel-header")).toBeFalsy();
  });

  it("adds interactive class when interactive prop is true", () => {
    const view = render(Panel, { props: { interactive: true, children: "Clickable" } });
    expect(view.container.querySelector(".interactive")).toBeTruthy();
  });
});

describe("StatusRow", () => {
  it("renders label and value", () => {
    const view = render(StatusRow, { props: { label: "Status", value: "Running" } });
    expect(view.getByText("Status")).toBeTruthy();
    expect(view.getByText("Running")).toBeTruthy();
  });

  it("defaults to muted tone", () => {
    const view = render(StatusRow, { props: { label: "L", value: "V" } });
    expect(view.container.querySelector(".tone-muted")).toBeTruthy();
  });

  it("applies primary tone", () => {
    const view = render(StatusRow, { props: { label: "L", value: "V", tone: "primary" } });
    expect(view.container.querySelector(".tone-primary")).toBeTruthy();
  });

  it("applies error tone", () => {
    const view = render(StatusRow, { props: { label: "L", value: "V", tone: "error" } });
    expect(view.container.querySelector(".tone-error")).toBeTruthy();
  });
});

describe("EmptyState", () => {
  it("renders eyebrow, title, and description", () => {
    const view = render(EmptyState, { props: {
      eyebrow: "WELCOME", title: "Get Started", description: "Configure your workspace",
    }});
    expect(view.getByText("WELCOME")).toBeTruthy();
    expect(view.getByText("Get Started")).toBeTruthy();
    expect(view.getByText("Configure your workspace")).toBeTruthy();
  });

  it("renders action button when actionLabel and onAction provided", async () => {
    const onAction = vi.fn();
    const view = render(EmptyState, { props: {
      eyebrow: "A", title: "T", description: "D",
      actionLabel: "Do it", onAction,
    }});
    await fireEvent.click(view.getByRole("button", { name: "Do it" }));
    expect(onAction).toHaveBeenCalledOnce();
  });

  it("keeps navigation separate from onboarding completion", async () => {
    const onAction = vi.fn();
    const onCompleteOnboarding = vi.fn();
    const view = render(EmptyState, { props: {
      eyebrow: "FIRST RUN", title: "Build your workspace", description: "Create zones",
      actionLabel: "Create your first layout", completionLabel: "Finish setup", onboarding: true,
      onAction, onCompleteOnboarding,
    }});
    await fireEvent.click(view.getByRole("button", { name: "Create your first layout" }));
    expect(onAction).toHaveBeenCalledOnce();
    expect(onCompleteOnboarding).not.toHaveBeenCalled();
    await fireEvent.click(view.getByRole("button", { name: "Finish setup" }));
    expect(onCompleteOnboarding).toHaveBeenCalledOnce();
  });

  it("does not render completion button when onboarding is false", () => {
    const onCompleteOnboarding = vi.fn();
    const view = render(EmptyState, { props: {
      eyebrow: "A", title: "T", description: "D",
      completionLabel: "Finish", onCompleteOnboarding, onboarding: false,
    }});
    expect(view.queryByRole("button", { name: "Finish" })).toBeFalsy();
  });
});

describe("ErrorPanel", () => {
  it("renders title and message", () => {
    const view = render(ErrorPanel, { props: { title: "Error", message: "Something went wrong" } });
    expect(view.getByText("Error")).toBeTruthy();
    expect(view.getByText("Something went wrong")).toBeTruthy();
  });

  it("renders Retry button when retry callback is supplied", async () => {
    const retry = vi.fn();
    const view = render(ErrorPanel, { props: { title: "E", message: "M", retry } });
    await fireEvent.click(view.getByRole("button", { name: "Retry" }));
    expect(retry).toHaveBeenCalledOnce();
  });

  it("does not render Retry button when retry is omitted", () => {
    const view = render(ErrorPanel, { props: { title: "E", message: "M" } });
    expect(view.queryByRole("button", { name: "Retry" })).toBeFalsy();
  });

  it("has alert role for accessibility", () => {
    const view = render(ErrorPanel, { props: { title: "E", message: "M" } });
    expect(view.getByRole("alert")).toBeTruthy();
  });
});
