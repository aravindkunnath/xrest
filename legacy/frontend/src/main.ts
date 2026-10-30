import { createApp } from "vue";
import App from "./app/App.vue";
import router from "./app/router";
import pinia from "./stores";
import "@fontsource-variable/inter/index.css";
import "@fontsource-variable/roboto-mono/index.css";
import "./app/style.css";

const app = createApp(App);
app.use(router);
app.use(pinia);
app.mount("#app");


