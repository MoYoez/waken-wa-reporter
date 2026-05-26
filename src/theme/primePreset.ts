import Aura from "@primevue/themes/aura";
import { definePreset } from "@primevue/themes";

export const WakenWaPreset = definePreset(Aura, {
  primitive: {
    sky: {
      50: "#eef4fb",
      100: "#dce8f6",
      200: "#b6cdec",
      300: "#86acdf",
      400: "#5b8bd0",
      500: "#3a6cbe",
      600: "#2755a4",
      700: "#1d4385",
      800: "#163568",
      900: "#122a52",
      950: "#0a1a36",
    },
    slate: {
      0: "#ffffff",
      50: "#f6f8fb",
      100: "#eef2f7",
      200: "#dde3ec",
      300: "#c2cad6",
      400: "#9aa4b3",
      500: "#6f7a8a",
      600: "#525c6c",
      700: "#3b4452",
      800: "#2a313c",
      900: "#181d25",
      950: "#0d1118",
    },
  },
  semantic: {
    primary: {
      50: "{sky.50}",
      100: "{sky.100}",
      200: "{sky.200}",
      300: "{sky.300}",
      400: "{sky.400}",
      500: "{sky.500}",
      600: "{sky.600}",
      700: "{sky.700}",
      800: "{sky.800}",
      900: "{sky.900}",
      950: "{sky.950}",
    },
    colorScheme: {
      light: {
        surface: {
          0: "#ffffff",
          50: "#f6f8fb",
          100: "#eef2f7",
          200: "#dde3ec",
          300: "#c2cad6",
          400: "#9aa4b3",
          500: "#6f7a8a",
          600: "#525c6c",
          700: "#3b4452",
          800: "#2a313c",
          900: "#181d25",
          950: "#0d1118",
        },
      },
    },
    formField: {
      borderRadius: "8px",
    },
    focusRing: {
      width: "2px",
      style: "solid",
      color: "rgba(39, 85, 164, 0.45)",
      offset: "2px",
      shadow: "0 0 0 0.18rem rgba(39, 85, 164, 0.18)",
    },
  },
});
