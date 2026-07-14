import { writable } from "svelte/store";
import type { ScreenInfo } from "../shared-types";

export const screens = writable<ScreenInfo[]>([]);
export const selectedScreenId = writable<string>("");
