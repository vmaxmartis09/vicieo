import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./features/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
          elevated: "hsl(var(--card-elevated))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
          hover: "hsl(var(--primary-hover))",
          soft: "var(--primary-soft)",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        brand: "hsl(var(--brand))",
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        semantic: {
          success: "hsl(var(--success))",
          warning: "hsl(var(--warning))",
          danger: "hsl(var(--danger))",
          info: "hsl(var(--info))",
        },
        edge: {
          text: "hsl(var(--edge-text))",
          image: "hsl(var(--edge-image))",
          video: "hsl(var(--edge-video))",
          audio: "hsl(var(--edge-audio))",
        },
      },
      fontFamily: {
        sans: ["var(--font-sans)", "Inter", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", "ui-monospace", "monospace"],
      },
      fontSize: {
        caption: ["11px", { lineHeight: "14px" }],
        footnote: ["12px", { lineHeight: "16px" }],
        body: ["13px", { lineHeight: "18px" }],
        callout: ["14px", { lineHeight: "20px" }],
        headline: ["15px", { lineHeight: "20px", fontWeight: "600" }],
        title: ["17px", { lineHeight: "22px" }],
        largetitle: ["22px", { lineHeight: "28px" }],
      },
      borderRadius: {
        xs: "var(--radius-xs, 4px)",
        ctl: "var(--radius-ctl, 8px)",
        card: "var(--radius-card, 12px)",
        panel: "var(--radius-panel, 16px)",
        sheet: "var(--radius-sheet, 24px)",
        sm: "var(--radius-sm, 4px)",
        md: "var(--radius-md, 8px)",
        lg: "var(--radius-lg, 8px)",
        xl: "var(--radius-xl, 16px)",
      },
      boxShadow: {
        subtle: "var(--shadow-sm)",
        card: "var(--shadow-md)",
        float: "var(--shadow-float)",
      },
      transitionTimingFunction: {
        springy: "cubic-bezier(0.16, 1, 0.3, 1)",
      },
      animation: {
        "pulse-subtle": "pulse 2.5s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },
    },
  },
  plugins: [],
};

export default config;
