import "./lib/theme.css";
import { mount } from "svelte";
import App from "./App.svelte";

const target = document.getElementById("app");

if (!target) {
  throw new Error("Grid Screen could not find its application root");
}

const app = mount(App, { target });

export default app;
