/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/renderer/index.html",
    "./src/renderer/app.ts",
    "./src/renderer/App.vue",
    "./src/renderer/**/*.{vue,js,ts,jsx,tsx}",
    "./src/renderer/components/**/*.{vue,js,ts}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        primary: "#ff3333", // 主题标志红
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
