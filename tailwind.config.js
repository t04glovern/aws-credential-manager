/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#005f73',
        secondary: '#0a9396',
        accent: '#94d2bd',
        neutral: '#e9d8a6',
        base: '#ee9b00',
        info: '#cae8d5',
        success: '#9b2226',
        warning: '#bb3e03',
        error: '#ae2012',
      },
    },
  },
  plugins: [],
}