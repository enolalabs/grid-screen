import { writable } from "svelte/store";
import type { WindowDescriptor } from "../shared-types";

export const windows = writable<WindowDescriptor[]>([]);
