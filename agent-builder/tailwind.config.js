/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: ['class', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        // N8N color palette
        canvas: {
          bg: '#f6f6f6',
          'bg-dark': '#2d2e2e',
        },
        node: {
          bg: '#ffffff',
          'bg-dark': '#3a3a3a',
          border: '#ddd',
          'border-dark': '#555',
          selected: '#ff6d5a',
        }
      },
    },
  },
  plugins: [],
}

