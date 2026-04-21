import Aura from "@primevue/themes/aura";
import { definePreset } from "@primevue/themes";

export const WakenWaPreset = definePreset(Aura, {
  primitive: {
    sky: {
      50: "#f3f9ff",
      100: "#e8f4ff",
      200: "#d6ebff",
      300: "#b9dcff",
      400: "#91caff",
      500: "#5db4ff",
      600: "#2d9bf0",
      700: "#1f7fcb",
      800: "#1d669f",
      900: "#1d557f",
      950: "#16324f",
    },
    slate: {
      0: "#ffffff",
      50: "#f8fbff",
      100: "#f1f7ff",
      200: "#e6eff8",
      300: "#cfdceb",
      400: "#a8bdd3",
      500: "#8199b2",
      600: "#647d97",
      700: "#4b647d",
      800: "#36506a",
      900: "#23405a",
      950: "#16324f",
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
          50: "#f8fbff",
          100: "#f1f7ff",
          200: "#e4effc",
          300: "#d4e4f6",
          400: "#bfd6ee",
          500: "#9bb8d7",
          600: "#7898bb",
          700: "#5a7c9f",
          800: "#40617f",
          900: "#2a4a66",
          950: "#16324f",
        },
      },
    },
    formField: {
      borderRadius: "18px",
    },
    focusRing: {
      width: "2px",
      style: "solid",
      color: "rgba(45, 155, 240, 0.42)",
      offset: "2px",
      shadow: "0 0 0 0.25rem rgba(45, 155, 240, 0.18)",
    },
  },
});
