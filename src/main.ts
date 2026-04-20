import { createApp } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";

import App from "@/App.vue";
import { i18n } from "@/i18n";
import { WakenWaPreset } from "@/theme/primePreset";
import "@/styles.css";
import "primeicons/primeicons.css";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(i18n);
app.use(PrimeVue, {
  ripple: true,
  inputVariant: "filled",
  theme: {
    preset: WakenWaPreset,
    options: {
      darkModeSelector: false,
    },
  },
});
app.use(ToastService);
app.mount("#app");
