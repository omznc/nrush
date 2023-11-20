# nrush 🦀

> A super fast way to update all packages in a Node/Bun project. Less featured, less tested, and less reliable alternative to npm-check-updates, but hey, it's blazingly fast 🔥

## Usage

Bun
```
bunx nrush
```

Node
```
npx nrush
```

Arguments:
  - `-u` / `--update` - Updates all dependencies without any further user interaction.
  - `-i` / `--interactive` - Will default to this if both `-u` and `-i` are supplied. Work in progress, does nothing right now. 
  - `-d` / `--dev` - Updates devDependencies as well. Can be combined with the above.


## Purpose 
I made this as a personal alternative to `npm-check-updates`, mostly as a challenge to write a less feature-packed, faster version that checks the packages concurrently, resulting in 🔥speed🔥.

From my testing, here are the differences (same project, ~60 dependencies)
- `bunx npm-check-updates -u` takes ~30 seconds
- `bunx nrush -u` takes ~2 seconds

Yes, that's more than 90% faster. Concurrency baby.

# Contributing
Please do. I don't really do Rust that often, and all of this was done in 30 minutes.

# Notes
- nrush is short for npm rush, as in "please get me up to date, I want the bleeding edge stuff and I love suffering"
- this completely ignores version ranges set in your `package.json`. Watch out.
