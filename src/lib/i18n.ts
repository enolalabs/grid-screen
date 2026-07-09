import { register, init, getLocaleFromNavigator, locale } from "svelte-i18n";
import { getSettings } from "./ipc";

register("en", () => import("./i18n/en.json"));
register("vi", () => import("./i18n/vi.json"));

export async function initI18n() {
  try {
    const settings = await getSettings();
    const lang = settings.language || getLocaleFromNavigator()?.split("-")[0] || "en";
    await init({ fallbackLocale: "en", initialLocale: lang });
  } catch {
    await init({ fallbackLocale: "en", initialLocale: "en" });
  }
}

export { locale };
