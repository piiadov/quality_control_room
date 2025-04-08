export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        background: 'var(--background-color)',
        backgroundSecondary: 'var(--background-secondary-color)',
        primary: 'var(--primary-color)',
        secondary: 'var(--secondary-color)',
        text: 'var(--text-color)',
        mutedText: 'var(--muted-text-color)',
        border: 'var(--border-color)',
        buttonBg: 'var(--button-bg)',
        buttonHoverBg: 'var(--button-hover-bg)',
        error: 'var(--error-color)',
        info: 'var(--info-color)',
      },
      fontFamily: {
        sans: ['Inter', 'Arial', 'sans-serif'],
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
