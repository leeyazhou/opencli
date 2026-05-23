/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/renderer/index.html",
    "./src/renderer/app.ts",
    "./src/renderer/App.vue",
    "./src/renderer/**/*.{vue,js,ts,jsx,tsx}",
    "./src/renderer/components/**/*.{vue,js,ts}",
  ],
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        background: "var(--color-background)",
        foreground: "var(--color-foreground)",
        muted: "var(--color-muted)",
        mutedForeground: "var(--color-muted-foreground)",
        border: "var(--color-border)",
        panel: "var(--color-panel)",
        panelBorder: "var(--color-panel-border)",
        sidebar: "var(--bg-sidebar)",
        activity: "var(--bg-activity)",
        primary: "var(--color-primary)",
        primaryForeground: "var(--color-primary-foreground)",
        bgGlass: "rgba(10, 10, 10, 0.4)",
        borderGlass: "rgba(255, 255, 255, 0.08)",
      },
      backdropBlur: {
        md: "20px",
      }
    },
  },
  plugins: [],
}
