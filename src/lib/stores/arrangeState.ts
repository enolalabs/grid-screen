import { writable } from "svelte/store";

export type ArrangeStatus =
  | { status: "idle" }
  | { status: "validating" }
  | { status: "arranging"; current: number; total: number }
  | { status: "completed"; errors: number }
  | { status: "failed"; reason: string };

export const arrangeState = writable<ArrangeStatus>({ status: "idle" });
