import { mount } from "svelte";
import "@fontsource-variable/inter";
import App from "./App.svelte";
import "./app.css";

export default mount(App, { target: document.getElementById("app")! });
