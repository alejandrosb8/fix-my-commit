# Write better commits

This is a Rust-based CLI tool designed to help write better-structured and more effective commits.

## Installation

You can install `fmc` by running the following command in your terminal.

```
curl -fsSL https://raw.githubusercontent.com/alejandrosb8/fix-my-commit/main/install.sh | sh -
```

## Usage

`fmc` uses [Cohere API](https://cohere.ai/). To use it, you'll need to grab an API key from [Cohere](https://cohere.ai/), and save it to `COHERE_API_KEY` as follows (you can also save it in your bash/zsh profile for persistance between sessions).

```bash
export COHERE_API_KEY='XXXXXXXX'
```

Once you have configured your environment, run `fmc`, it needs two parameter, -p (or --prefix) and -m (or --message)

Prefixes that you can use: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `revert`, `build`, `ci`.
