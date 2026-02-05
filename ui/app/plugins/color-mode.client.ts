export default defineNuxtPlugin(() => {
    const applyTheme = (isDark: boolean) => {
        document.documentElement.classList.toggle("dark", isDark);
    };

    const stored = localStorage.getItem("theme");
    if (stored === "dark" || stored === "light") {
        applyTheme(stored === "dark");
        return;
    }

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    applyTheme(media.matches);

    const onChange = (event: MediaQueryListEvent) => {
        applyTheme(event.matches);
    };

    if (media.addEventListener) {
        media.addEventListener("change", onChange);
    } else {
        media.addListener(onChange);
    }
});
