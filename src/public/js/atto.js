console.log("🐐 tetratto - https://github.com/trisuaso/tetratto");

// theme preference
function media_theme_pref() {
    document.documentElement.removeAttribute("class");

    if (
        window.matchMedia("(prefers-color-scheme: dark)").matches &&
        !window.localStorage.getItem("tetratto:theme")
    ) {
        document.documentElement.classList.add("dark");
        // window.localStorage.setItem("theme", "dark");
    } else if (
        window.matchMedia("(prefers-color-scheme: light)").matches &&
        !window.localStorage.getItem("tetratto:theme")
    ) {
        document.documentElement.classList.remove("dark");
        // window.localStorage.setItem("theme", "light");
    } else if (window.localStorage.getItem("tetratto:theme")) {
        /* restore theme */
        const current = window.localStorage.getItem("tetratto:theme");
        document.documentElement.className = current;
    }
}

function set_theme(theme) {
    window.localStorage.setItem("tetratto:theme", theme);
    document.documentElement.className = theme;
}

media_theme_pref();
