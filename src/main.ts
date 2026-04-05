import { createApp } from "vue";
import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";
import Aura from "@primevue/themes/aura";
import { definePreset } from "@primevue/themes";

import App from "./App.vue";
import "./styles.css";
import "primeicons/primeicons.css";

const WakenWaPreset = definePreset(Aura, {
  primitive: {
    amber: {
      50: "#fff8ef",
      100: "#fef0dc",
      200: "#f9ddb4",
      300: "#f2c689",
      400: "#e7a95b",
      500: "#cc8440",
      600: "#b56c34",
      700: "#935329",
      800: "#784326",
      900: "#623921",
      950: "#351d10",
    },
    slate: {
      0: "#ffffff",
      50: "#faf7f2",
      100: "#f2ece2",
      200: "#e4d8c9",
      300: "#cfbaa2",
      400: "#b19475",
      500: "#96785a",
      600: "#7b6049",
      700: "#654f3c",
      800: "#544233",
      900: "#46372b",
      950: "#261d17",
    },
  },
  semantic: {
    primary: {
      50: "{amber.50}",
      100: "{amber.100}",
      200: "{amber.200}",
      300: "{amber.300}",
      400: "{amber.400}",
      500: "{amber.500}",
      600: "{amber.600}",
      700: "{amber.700}",
      800: "{amber.800}",
      900: "{amber.900}",
      950: "{amber.950}",
    },
    colorScheme: {
      light: {
        surface: {
          0: "#fffdf8",
          50: "#faf6ef",
          100: "#f4ede2",
          200: "#e9dcc7",
          300: "#d8c2a2",
          400: "#bb9c73",
          500: "#9f7e57",
          600: "#856347",
          700: "#6d503b",
          800: "#5a4130",
          900: "#493426",
          950: "#271b14",
        },
      },
    },
    formField: {
      borderRadius: "18px",
    },
    focusRing: {
      width: "2px",
      style: "solid",
      color: "rgba(204, 132, 64, 0.42)",
      offset: "2px",
      shadow: "0 0 0 0.25rem rgba(204, 132, 64, 0.18)",
    },
  },
});

const app = createApp(App);

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
