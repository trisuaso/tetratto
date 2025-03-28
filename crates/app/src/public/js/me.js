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
})();
