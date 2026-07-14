import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "@fontsource/inter/700.css";
import App from "./App.svelte";
import "./app.css";

const app = new App({
  target: document.getElementById("app")!,
});

export default app;
