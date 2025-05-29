/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        'primary': ['Indie Flower', 'cursive'],
        'secondary': ['Noto Sans', 'sans-serif'],
      },
    },
  },
  plugins: [],
} 