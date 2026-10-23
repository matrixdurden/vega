// Only the subsets and weight the UI uses: latin plus latin-ext for Turkish.
import "@fontsource/pixelify-sans/latin-600.css";
import "@fontsource/pixelify-sans/latin-ext-600.css";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

export default mount(App, { target: document.getElementById("app")! });
