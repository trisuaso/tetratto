(() => {
    const self = reg_ns("me");

    self.LOGIN_ACCOUNT_TOKENS = JSON.parse(
        window.localStorage.getItem("atto:login_account_tokens") || "{}",
    );

    self.define("logout", async () => {
        if (
            !(await trigger("atto::confirm", [
                "Are you sure you would like to do this?",
            ]))
        ) {
            return;
        }

        fetch("/api/v1/auth/logout", {
            method: "POST",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);

                if (res.ok) {
                    setTimeout(() => {
                        window.location.href = "/";
                    }, 150);
                }
            });
    });

    self.define("remove_post", async (_, id) => {
        if (
            !(await trigger("atto::confirm", [
                "Are you sure you want to do this?",
            ]))
        ) {
            return;
        }

        fetch(`/api/v1/posts/${id}`, {
            method: "DELETE",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    self.define("react", async (_, element, asset, asset_type, is_like) => {
        fetch("/api/v1/reactions", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                asset,
                asset_type,
                is_like,
            }),
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);

                if (res.ok) {
                    const like = element.parentElement.querySelector(
                        '[hook_element="reaction.like"]',
                    );

                    const dislike = element.parentElement.querySelector(
                        '[hook_element="reaction.dislike"]',
                    );

                    if (is_like) {
                        like.classList.add("green");
                        like.querySelector("svg").classList.add("filled");

                        dislike.classList.remove("red");
                    } else {
                        dislike.classList.add("red");

                        like.classList.remove("green");
                        like.querySelector("svg").classList.remove("filled");
                    }
                }
            });
    });

    self.define("remove_notification", (_, id) => {
        fetch(`/api/v1/notifications/${id}`, {
            method: "DELETE",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    self.define("update_notification_read_status", (_, id, read) => {
        fetch(`/api/v1/notifications/${id}/read_status`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                read,
            }),
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    self.define("clear_notifs", async () => {
        if (
            !(await trigger("atto::confirm", [
                "Are you sure you want to do this?",
            ]))
        ) {
            return;
        }

        fetch("/api/v1/notifications/my", {
            method: "DELETE",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    self.define("repost", (_, id, content, community) => {
        fetch(`/api/v1/posts/${id}/repost`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                content,
                community,
            }),
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);

                if (res.ok) {
                    setTimeout(() => {
                        window.location.href = `/post/${res.payload}`;
                    }, 100);
                }
            });
    });

    self.define("report", (_, asset, asset_type) => {
        window.open(
            `/mod_panel/file_report?asset=${asset}&asset_type=${asset_type}`,
        );
    });

    self.define("seen", () => {
        fetch("/api/v1/auth/user/me/seen", {
            method: "POST",
        })
            .then((res) => res.json())
            .then((res) => {
                if (!res.ok) {
                    trigger("atto::toast", ["error", res.message]);
                }
            });
    });

    self.define("remove_question", async (_, id) => {
        if (
            !(await trigger("atto::confirm", [
                "Are you sure you want to do this?",
            ]))
        ) {
            return;
        }

        fetch(`/api/v1/questions/${id}`, {
            method: "DELETE",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    self.define("ip_block_question", async (_, id) => {
        if (
            !(await trigger("atto::confirm", [
                "Are you sure you want to do this?",
            ]))
        ) {
            return;
        }

        fetch(`/api/v1/questions/${id}/block_ip`, {
            method: "POST",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("atto::toast", [
                    res.ok ? "success" : "error",
                    res.message,
                ]);
            });
    });

    // token switcher
    self.define(
        "set_login_account_tokens",
        ({ $ }, value) => {
            $.LOGIN_ACCOUNT_TOKENS = value;
            window.localStorage.setItem(
                "atto:login_account_tokens",
                JSON.stringify(value),
            );
        },
        ["object"],
    );

    self.define("login", ({ $ }, username) => {
        const token = self.LOGIN_ACCOUNT_TOKENS[username];

        if (!token) {
            return;
        }

        window.location.href = `/api/v1/auth/token?token=${token}`;
    });

    self.define("render_token_picker", ({ $ }, element) => {
        element.innerHTML = "";
        for (const token of Object.entries($.LOGIN_ACCOUNT_TOKENS)) {
            element.innerHTML += `<button class="quaternary w-full justify-start" onclick="trigger('me::login', ['${token[0]}'])">
                <img
                    title="${token[0]}'s avatar"
                    src="/api/v1/auth/user/${token[0]}/avatar?selector_type=username"
                    alt="Avatar image"
                    class="avatar"
                    style="--size: 24px"
                />

                <span>${token[0]}</span>
            </button>`;
        }
    });

    self.define("switch_account", () => {
        document.getElementById("tokens_dialog").showModal();
    });
})();
