(() => {
    const self = reg_ns("me");

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
})();
