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

// atto ns
(() => {
    const self = reg_ns("atto");

    // init
    use("me", () => {});

    // env
    self.DEBOUNCE = [];
    self.OBSERVERS = [];

    // ...
    self.define("try_use", (_, ns_name, callback) => {
        // attempt to get existing namespace
        if (globalThis._app_base.ns_store[`$${ns_name}`]) {
            return callback(globalThis._app_base.ns_store[`$${ns_name}`]);
        }

        // otherwise, call normal use
        use(ns_name, callback);
    });

    self.define("debounce", ({ $ }, name) => {
        return new Promise((resolve, reject) => {
            if ($.DEBOUNCE.includes(name)) {
                return reject();
            }

            $.DEBOUNCE.push(name);

            setTimeout(() => {
                delete $.DEBOUNCE[$.DEBOUNCE.indexOf(name)];
            }, 1000);

            return resolve();
        });
    });

    self.define("rel_date", (_, date) => {
        // stolen and slightly modified because js dates suck
        const diff = (new Date().getTime() - date.getTime()) / 1000;
        const day_diff = Math.floor(diff / 86400);

        if (Number.isNaN(day_diff) || day_diff < 0 || day_diff >= 31) {
            return;
        }

        return (
            (day_diff === 0 &&
                ((diff < 60 && "just now") ||
                    (diff < 120 && "1 minute ago") ||
                    // biome-ignore lint/style/useTemplate: ok
                    (diff < 3600 && Math.floor(diff / 60) + " minutes ago") ||
                    (diff < 7200 && "1 hour ago") ||
                    (diff < 86400 &&
                        // biome-ignore lint/style/useTemplate: ok
                        Math.floor(diff / 3600) + " hours ago"))) ||
            (day_diff === 1 && "Yesterday") ||
            // biome-ignore lint/style/useTemplate: ok
            (day_diff < 7 && day_diff + " days ago") ||
            // biome-ignore lint/style/useTemplate: ok
            (day_diff < 31 && Math.ceil(day_diff / 7) + " weeks ago")
        );
    });

    self.define("clean_date_codes", ({ $ }) => {
        for (const element of Array.from(document.querySelectorAll(".date"))) {
            if (element.getAttribute("data-unix")) {
                // this allows us to run the function twice on the same page
                // without errors from already rendered dates
                element.innerText = element.getAttribute("data-unix");
            }

            element.setAttribute("data-unix", element.innerText);
            const then = new Date(Number.parseInt(element.innerText));

            if (Number.isNaN(element.innerText)) {
                continue;
            }

            element.setAttribute("title", then.toLocaleString());

            let pretty = $.rel_date(then);

            if (screen.width < 900 && pretty !== undefined) {
                // shorten dates even more for mobile
                pretty = pretty
                    .replaceAll(" minutes ago", "m")
                    .replaceAll(" minute ago", "m")
                    .replaceAll(" hours ago", "h")
                    .replaceAll(" hour ago", "h")
                    .replaceAll(" days ago", "d")
                    .replaceAll(" day ago", "d")
                    .replaceAll(" weeks ago", "w")
                    .replaceAll(" week ago", "w")
                    .replaceAll(" months ago", "m")
                    .replaceAll(" month ago", "m")
                    .replaceAll(" years ago", "y")
                    .replaceAll(" year ago", "y");
            }

            element.innerText =
                pretty === undefined ? then.toLocaleDateString() : pretty;

            element.style.display = "inline-block";
        }
    });

    self.define("copy_text", ({ $ }, text) => {
        navigator.clipboard.writeText(text);
        $.toast("success", "Copied!");
    });

    self.define("smooth_remove", (_, element, ms) => {
        // run animation
        element.style.animation = `fadeout ease-in-out 1 ${ms}ms forwards running`;

        // remove
        setTimeout(() => {
            element.remove();
        }, ms);
    });

    self.define("disconnect_observers", ({ $ }) => {
        for (const observer of $.OBSERVERS) {
            observer.disconnect();
        }

        $.OBSERVERS = [];
    });

    self.define("offload_work_to_client_when_in_view", (_, entry_callback) => {
        // instead of spending the time on the server loading everything before
        // returning the page, we can instead of just create an IntersectionObserver
        // and send individual requests as we see the element it's needed for
        const seen = [];
        return new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    const element = entry.target;
                    if (!entry.isIntersecting || seen.includes(element)) {
                        continue;
                    }

                    seen.push(element);
                    entry_callback(element);
                }
            },
            {
                root: document.body,
                rootMargin: "0px",
                threshold: 1.0,
            },
        );
    });

    self.define("toggle_flex", (_, element) => {
        if (element.style.display === "none") {
            element.style.display = "flex";
        } else {
            element.style.display = "none";
        }
    });

    // hooks
    self.define("hooks::scroll", (_, scroll_element, track_element) => {
        const goals = [150, 250, 500, 1000];

        track_element.setAttribute("data-scroll", "0");
        scroll_element.addEventListener("scroll", (e) => {
            track_element.setAttribute("data-scroll", scroll_element.scrollTop);

            for (const goal of goals) {
                const name = `data-scroll-${goal}`;
                if (scroll_element.scrollTop >= goal) {
                    track_element.setAttribute(name, "true");
                } else {
                    track_element.removeAttribute(name);
                }
            }
        });
    });

    self.define("hooks::dropdown.close", (_) => {
        for (const dropdown of Array.from(
            document.querySelectorAll(".inner.open"),
        )) {
            dropdown.classList.remove("open");
        }
    });

    self.define("hooks::dropdown", ({ $ }, event) => {
        event.stopImmediatePropagation();
        let target = event.target;

        while (!target.matches(".dropdown")) {
            target = target.parentElement;
        }

        // close all others
        $["hooks::dropdown.close"]();

        // open
        setTimeout(() => {
            for (const dropdown of Array.from(
                target.querySelectorAll(".inner"),
            )) {
                // check y
                const box = target.getBoundingClientRect();

                let parent = dropdown.parentElement;

                while (!parent.matches("html, .window")) {
                    parent = parent.parentElement;
                }

                let parent_height = parent.getBoundingClientRect().y;

                if (parent.nodeName === "HTML") {
                    parent_height = window.screen.height;
                }

                const scroll = window.scrollY;
                const height = parent_height;
                const y = box.y + scroll;

                if (y > height - scroll - 300) {
                    dropdown.classList.add("top");
                } else {
                    dropdown.classList.remove("top");
                }

                // open
                dropdown.classList.add("open");

                if (dropdown.classList.contains("open")) {
                    dropdown.removeAttribute("aria-hidden");
                } else {
                    dropdown.setAttribute("aria-hidden", "true");
                }
            }
        }, 5);
    });

    self.define("hooks::dropdown.init", (_, bind_to) => {
        for (const dropdown of Array.from(
            document.querySelectorAll(".inner"),
        )) {
            dropdown.setAttribute("aria-hidden", "true");
        }

        bind_to.addEventListener("click", (event) => {
            if (
                event.target.matches(".dropdown") ||
                event.target.matches("[exclude=dropdown]")
            ) {
                return;
            }

            for (const dropdown of Array.from(
                document.querySelectorAll(".inner.open"),
            )) {
                dropdown.classList.remove("open");
            }
        });
    });

    self.define("hooks::character_counter", (_, event) => {
        let target = event.target;

        while (!target.matches("textarea, input")) {
            target = target.parentElement;
        }

        const counter = document.getElementById(`${target.id}:counter`);
        counter.innerText = `${target.value.length}/${target.getAttribute("maxlength")}`;
    });

    self.define("hooks::character_counter.init", (_, event) => {
        for (const element of Array.from(
            document.querySelectorAll("[hook=counter]") || [],
        )) {
            const counter = document.getElementById(`${element.id}:counter`);
            counter.innerText = `0/${element.getAttribute("maxlength")}`;
            element.addEventListener("keyup", (e) =>
                app["hooks::character_counter"](e),
            );
        }
    });

    self.define("hooks::long", (_, element, full_text) => {
        element.classList.remove("hook:long.hidden_text");
        element.innerHTML = full_text;
    });

    self.define("hooks::long_text.init", (_, event) => {
        for (const element of Array.from(
            document.querySelectorAll("[hook=long]") || [],
        )) {
            const is_long = element.innerText.length >= 64 * 16;

            if (!is_long) {
                continue;
            }

            element.classList.add("hook:long.hidden_text");

            if (element.getAttribute("hook-arg") === "lowered") {
                element.classList.add("hook:long.hidden_text+lowered");
            }

            const html = element.innerHTML;
            const short = html.slice(0, 64 * 16);
            element.innerHTML = `${short}...`;

            // event
            const listener = () => {
                app["hooks::long"](element, html);
                element.removeEventListener("click", listener);
            };

            element.addEventListener("click", listener);
        }
    });

    self.define("hooks::alt", (_) => {
        for (const element of Array.from(
            document.querySelectorAll("img") || [],
        )) {
            if (element.getAttribute("alt") && !element.getAttribute("title")) {
                element.setAttribute("title", element.getAttribute("alt"));
            }
        }
    });

    self.define(
        "hooks::attach_to_partial",
        ({ $ }, partial, full, attach, wrapper, page, run_on_load) => {
            return new Promise((resolve, reject) => {
                async function load_partial() {
                    const url = `${partial}${partial.includes("?") ? "&" : "?"}page=${page}`;
                    history.replaceState(
                        history.state,
                        "",
                        url.replace(partial, full),
                    );

                    fetch(url)
                        .then(async (res) => {
                            const text = await res.text();

                            if (
                                text.length < 100 ||
                                text.includes('data-marker="no-results"')
                            ) {
                                // pretty much blank content, no more pages
                                wrapper.removeEventListener("scroll", event);

                                return resolve();
                            }

                            attach.innerHTML += text;

                            $.clean_date_codes();
                            $.link_filter();
                            $["hooks::alt"]();
                        })
                        .catch(() => {
                            // done scrolling, no more pages (http error)
                            wrapper.removeEventListener("scroll", event);

                            resolve();
                        });
                }

                const event = async () => {
                    if (
                        wrapper.scrollTop + wrapper.offsetHeight + 100 >
                        attach.offsetHeight
                    ) {
                        self.debounce("app::partials")
                            .then(async () => {
                                if (document.getElementById("initial_loader")) {
                                    console.log("partial blocked");
                                    return;
                                }

                                // biome-ignore lint/style/noParameterAssign: no it isn't
                                page += 1;
                                await load_partial();
                                await $["hooks::partial_embeds"]();
                            })
                            .catch(() => {
                                console.log("partial stuck");
                            });
                    }
                };

                wrapper.addEventListener("scroll", event);
            });
        },
    );

    self.define("hooks::partial_embeds", (_) => {
        for (const paragraph of Array.from(
            document.querySelectorAll("span[class] p"),
        )) {
            const groups = /(\/\+r\/)([\w]+)/.exec(paragraph.innerText);

            if (groups === null) {
                continue;
            }

            // add embed
            paragraph.innerText = paragraph.innerText.replace(groups[0], "");
            paragraph.parentElement.innerHTML += `<include-partial
                   src="/_app/components/response.html?id=${groups[2]}&do_render_nested=false"
                   uses="app::clean_date_codes,app::link_filter,app::hooks::alt"
               ></include-partial>`;
        }
    });

    self.define("hooks::check_reactions", async ({ $ }) => {
        const observer = $.offload_work_to_client_when_in_view(
            async (element) => {
                const like = element.querySelector(
                    '[hook_element="reaction.like"]',
                );

                const dislike = element.querySelector(
                    '[hook_element="reaction.dislike"]',
                );

                const reaction = await (
                    await fetch(
                        `/api/v1/reactions/${element.getAttribute("hook-arg:id")}`,
                    )
                ).json();

                if (reaction.ok) {
                    if (reaction.payload.is_like) {
                        like.classList.add("green");
                        like.querySelector("svg").classList.add("filled");
                    } else {
                        dislike.classList.add("red");
                    }
                }
            },
        );

        for (const element of Array.from(
            document.querySelectorAll("[hook=check_reactions]") || [],
        )) {
            observer.observe(element);
        }

        $.OBSERVERS.push(observer);
    });

    self.define("hooks::tabs:switch", (_, tab) => {
        // tab
        for (const element of Array.from(
            document.querySelectorAll("[data-tab]"),
        )) {
            element.classList.add("hidden");
        }

        document
            .querySelector(`[data-tab="${tab}"]`)
            .classList.remove("hidden");

        // button
        if (document.querySelector(`[data-tab-button="${tab}"]`)) {
            for (const element of Array.from(
                document.querySelectorAll("[data-tab-button]"),
            )) {
                element.classList.remove("active");
            }

            document
                .querySelector(`[data-tab-button="${tab}"]`)
                .classList.add("active");
        }
    });

    self.define("hooks::tabs:check", ({ $ }, hash) => {
        if (!hash || !hash.startsWith("#/")) {
            return;
        }

        $["hooks::tabs:switch"](hash.replace("#/", ""));
    });

    self.define("hooks::tabs", ({ $ }) => {
        $["hooks::tabs:check"](window.location.hash); // initial check
        window.addEventListener("hashchange", (event) =>
            $["hooks::tabs:check"](new URL(event.newURL).hash),
        );
    });

    // web api replacements
    self.define("prompt", (_, msg) => {
        const dialog = document.getElementById("web_api_prompt");
        document.getElementById("web_api_prompt:msg").innerText = msg;

        return new Promise((resolve, _) => {
            globalThis.web_api_prompt_submit = (value) => {
                dialog.close();
                return resolve(value);
            };

            dialog.showModal();
        });
    });

    self.define("prompt_long", (_, msg) => {
        const dialog = document.getElementById("web_api_prompt_long");
        document.getElementById("web_api_prompt_long:msg").innerText = msg;

        return new Promise((resolve, _) => {
            globalThis.web_api_prompt_long_submit = (value) => {
                dialog.close();
                return resolve(value);
            };

            dialog.showModal();
        });
    });

    self.define("confirm", (_, msg) => {
        const dialog = document.getElementById("web_api_confirm");
        document.getElementById("web_api_confirm:msg").innerText = msg;

        return new Promise((resolve, _) => {
            globalThis.web_api_confirm_submit = (value) => {
                dialog.close();
                return resolve(value);
            };

            dialog.showModal();
        });
    });

    // toast
    self.define("toast", ({ $ }, type, content, time_until_remove = 5) => {
        const element = document.createElement("div");
        element.id = "toast";
        element.classList.add(type);
        element.classList.add("toast");
        element.innerHTML = `<span>${content
            .replaceAll("<", "&lt")
            .replaceAll(">", "&gt;")}</span>`;

        document.getElementById("toast_zone").prepend(element);

        const timer = document.createElement("span");
        element.appendChild(timer);

        timer.innerText = time_until_remove;
        timer.classList.add("timer");

        // start timer
        setTimeout(() => {
            clearInterval(count_interval);
            $.smooth_remove(element, 500);
        }, time_until_remove * 1000);

        const count_interval = setInterval(() => {
            // biome-ignore lint/style/noParameterAssign: no it isn't
            time_until_remove -= 1;
            timer.innerText = time_until_remove;
        }, 1000);
    });

    // link filter
    self.define("link_filter", (_) => {
        for (const anchor of Array.from(document.querySelectorAll("a"))) {
            if (anchor.href.length === 0) {
                continue;
            }

            const url = new URL(anchor.href);
            if (
                anchor.href.startsWith("/") ||
                anchor.href.startsWith("javascript:") ||
                url.origin === window.location.origin
            ) {
                continue;
            }

            anchor.addEventListener("click", (e) => {
                e.preventDefault();
                document.getElementById("link_filter_url").innerText =
                    anchor.href;
                document.getElementById("link_filter_continue").href =
                    anchor.href;
                document.getElementById("link_filter").showModal();
            });
        }
    });
})();

// ui ns
(() => {
    const self = reg_ns("ui");

    self.define("render_settings_ui_field", (_, into_element, option) => {
        into_element.innerHTML += `<div class="card-nest">
            <div class="card small">
                <b>${option.label.replaceAll("_", " ")}</b>
            </div>

            <div class="card">
                <${option.input_element_type || "input"}
                    type="text"
                    onchange="window.set_setting_field('${option.key}', event.target.value)"
                    placeholder="${option.key}"
                    ${option.input_element_type === "input" ? `value="${option.value}"/>` : ">"}
${option.input_element_type === "textarea" ? `${option.value}</textarea>` : ""}
            </div>
        </div>`;
    });

    self.define(
        "generate_settings_ui",
        ({ $ }, into_element, options, settings_ref) => {
            for (const option of options) {
                $.render_settings_ui_field(into_element, {
                    key: Array.isArray(option[0]) ? option[0][0] : option[0],
                    label: Array.isArray(option[0]) ? option[0][1] : option[0],
                    value: option[1],
                    input_element_type: option[2],
                });
            }

            window.set_setting_field = (key, value) => {
                settings_ref[key] = value;
                console.log("update", key);
            };
        },
    );
})();
