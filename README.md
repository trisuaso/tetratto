# 🐐 tetratto!

This is the year of the personal website.

Tetratto (`4 * 10^-18`) is a _super_ simple **dynamic** site server which takes in a conglomeration of HTML files (which are actually Jinja templates) and static files like CSS and JS, then serves them!

## Features

- Templated HTML files (`html/` directory)
- Markdown posts (`posts/` directory, served with `html/post.html` template)
- Super simple SQLite database for authentication (and other stuff)

## Usage

Install Tetratto CLI:

```bash
cargo install tetratto
```

Clone the `./example` directory to get started.

You can run a project by running `tetratto` in the directory. The entry file for CSS is assumed to be `public/css/style.css`. Note that your `index.html` file should _not_ include boilerplate stuff, and should instead just include a `{% block body %}` for beginning your content in the body. `{% block head %}` can be used to place data in the page head element. Templates should all extend the `_atto/root.html` template.

### Config

You can configure Tetratto by editing the project's `tetratto.toml` file.

- `name`: the `{{ name }}` variable in templates (default: `Tetratto`)
- `port`: the port the server is served on (default: `4118`)
- `database`: the name of the file to store the SQLite database in (default: `./atto.db`)
