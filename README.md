<div align="center">
    <h1>🐇 tetratto!</h1>
    <p><i>Tetratto</i> is a super simple community-oriented website where users can create various communities and share posts in them!</p>
</div>

!["Docs" workflow badge](https://github.com/trisuaso/tetratto/workflows/Docs/badge.svg)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/trisuaso/tetratto)
![GitHub last commit](https://img.shields.io/github/last-commit/trisuaso/tetratto)
[![GitHub License](https://img.shields.io/github/license/trisuaso/tetratto)](https://github.com/trisuaso/tetratto/blob/master/LICENSE)

# Usage

Everything Tetratto needs will be built into the main binary. You can build Tetratto with the following command:

```bash
cargo build -r --no-default-features --features=redis,sqlite
```

You can replace `sqlite` in the above command with `postgres`, if you'd like. It's also acceptable to remove the `redis` part if you don't want to use a cache. <sup>I wouldn't recomment removing cache, though</sup>

You can then take the binary and place it somewhere else (highly recommended; the binary will create a fair number of files!). You can do this to move it to a directory just called "tetratto" in the parent directory:

```bash
mkdir tetratto
mv ./target/release/tetratto ../tetratto
cd ../tetratto
```

Your first start of Tetratto might be a little slow as it's going to download all icon SVGs required for the HTML templates to render properly. These icons will be stored on disk, so there's no need to worry about this time _every_ restart.

## Configuration

In the directory you're running Tetratto from, you should create a `tetratto.toml` file. This file follows the configuration schema defined [here](https://trisuaso.github.io/tetratto/tetratto/config/struct.Config.html)!

Tetratto **requires** Cloudflare Turnstile for registrations. Testing keys are listed [here](https://developers.cloudflare.com/turnstile/troubleshooting/testing/). You can _technically_ disable the captcha by using the always passing, invisible keys.

## Usage (as a user)

Tetratto is very simple once you get the hang of it! At the top of the page (or bottom if you're on mobile), you'll see the navigation bar. Once logged in, you'll be able to access "Home", "Popular", and "Communities" from there! You can also press your profile picture (on the right) to view your own profile, settings, or log out!

All Tetratto instances support reports for communities and posts through the UI. You can just find the ellipsis icon on either and then press "Report" to file a report!

# Updating

When bumping versions, you _might_ need to run a few SQL scripts in order to get your database up to what the next commit expects. It is recommended to only update from GitHub release, which will list the SQL scripts you need to run to migrate your database.

# Developing

All you really need to develop Tetratto is [Rust](https://rustup.rs/) and [Just](https://just.systems/).

You can fix a lot of weird style issues and stuff using `just fix`. <sup>You need [Clippy](https://doc.rust-lang.org/stable/clippy/installation.html) for this</sup>

You can also automatically bump all dependencies _and_ point out unused ones with `just clean-deps`! <sup>You need [cargo-edit](https://github.com/killercup/cargo-edit) and [cargo-machete](https://github.com/bnjbvr/cargo-machete) for this</sup>

# Contributing

Read the ["Contribution Guidelines"](./.github/CONTRIBUTING.md) before contributing!

# License

Tetratto is licensed under the [AGPL-3.0](./LICENSE).
