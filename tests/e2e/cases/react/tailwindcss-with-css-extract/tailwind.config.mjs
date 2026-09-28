import path from 'node:path';
/** @type {import('tailwindcss').Config} */
export default {
  content: [path.join(import.meta.dirname, './src/**/*.{html,js,jsx}')],
  theme: {
    extend: {},
  },
  plugins: [],
};
