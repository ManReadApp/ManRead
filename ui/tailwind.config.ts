import type { Config } from "tailwindcss";

export default {
    darkMode: "class",
    content: [
        "./app/**/*.{vue,js,ts}",
        "./components/**/*.{vue,js,ts}",
        "./pages/**/*.{vue,js,ts}",
        "./layouts/**/*.{vue,js,ts}",
        "./plugins/**/*.{js,ts}",
        "./nuxt.config.{js,ts}",
    ],
    theme: {
        extend: {
            boxShadow: {
                soft: "0 12px 30px -18px rgba(15, 23, 42, 0.45)",
            },
        },
    },
    plugins: [],
} satisfies Config;
