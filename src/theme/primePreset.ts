import Aura from "@primevue/themes/aura";
import { definePreset } from "@primevue/themes";

export const WakenWaPreset = definePreset(Aura, {
  primitive: {
    sky: {
      50: "#eef7f7",
      100: "#dff1f1",
      200: "#c4e5e5",
      300: "#9bd2d3",
      400: "#63b9bb",
      500: "#2d9fa3",
      600: "#0f8b8d",
      700: "#0b6f72",
      800: "#0a5a5d",
      900: "#0a484b",
      950: "#163235",
    },
    slate: {
      0: "#ffffff",
      50: "#f7f8fa",
      100: "#f1f3f5",
      200: "#e3e7ec",
      300: "#c7d0da",
      400: "#a4acb8",
      500: "#747f8f",
      600: "#566273",
      700: "#3f4a59",
      800: "#2f3a49",
      900: "#1c2430",
      950: "#111827",
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
          50: "#f7f8fa",
          100: "#f1f3f5",
          200: "#e3e7ec",
          300: "#c7d0da",
          400: "#a4acb8",
          500: "#747f8f",
          600: "#566273",
          700: "#3f4a59",
          800: "#2f3a49",
          900: "#1c2430",
          950: "#111827",
        },
      },
    },
    formField: {
      borderRadius: "6px",
    },
    focusRing: {
      width: "2px",
      style: "solid",
      color: "rgba(15, 139, 141, 0.42)",
      offset: "2px",
      shadow: "0 0 0 0.18rem rgba(15, 139, 141, 0.14)",
    },
  },
});
