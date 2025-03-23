clean-deps:
    cargo upgrade -i
    cargo machete

fix:
    cargo fix --allow-dirty
    cargo clippy --fix --allow-dirty
