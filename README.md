<center>

![nrush](https://github.com/omznc/nrush/assets/38432561/ad2f9d0c-477a-420a-aa34-7c171fe8a0a8)

</center>

# nrush 🦀

> A super fast way to update all packages in a Node/Bun project. (probably) less featured, less tested, and less
> reliable alternative to npm-check-updates, but hey, it's blazingly fast 🔥

## Usage

Bun

```
bunx nrush@latest
```

Node

```
npx nrush@latest
```


Commands:
- `nrush about` - Show information about nrush.
- `nrush help` - Show a manual on how to use nrush. Basically this.

Arguments (only apply if no commands are supplied, only `nrush`):
- `-u` / `--update` - Updates all dependencies without any further user interaction.
- `-i` / `--interactive` - Pick and choose which packages, if any, to update. Will default to this if both `-u` and `-i`
  are supplied.
- `--include <dev,peer>` - Include dev and/or peer dependencies in the update process. Defaults to nothing, but you can
  add `dev` and/or `peer` to include them, comma separated.
- `-p <path>` / `--path <path>` - Specify a path to a package.json file. Defaults to the current directory.
- `--skip-ranges` - Skip version ranges in package.json. Defaults to `false`, and keeps them.
	- `^1.0.0` will be updated to `2.0.0` if `--skip-ranges` is supplied.
	- (default) `^1.0.0` will be updated to `^2.0.0` if `--skip-ranges` is **not** supplied.
- `--update-any` - Update `*` versions in package.json. Defaults to `false`, and keeps them.
	- `*` will be updated to `^2.0.0` if `--update-any` is supplied.
	- (default) `*` will not be touched if `--update-any` is **not** supplied.
- `-s <semver>` / `--semver <semver>` - Specify a maximum semver range to update to. Choose 1
  between `major`, `minor`, `patch`. Defaults to `major`.
	- Currently does nothing, but will be implemented soon.

Running `nrush` without any arguments or commands will show you a list of packages that can be updated.


## Purpose

I made this as a personal alternative to `npm-check-updates`, mostly as a challenge to write a less feature-packed,
faster version that checks the packages concurrently, resulting in 🔥speed🔥.

From my testing, here are the differences (same project, ~60 dependencies)

- `bunx npm-check-updates -u` takes ~30 seconds
- `bunx nrush -u` takes ~2 seconds

Yes, that's more than 90% faster. Concurrency baby.

# Contributing

Please do. I don't really do Rust that often, and all of this was done in 30 minutes.

# Goals
- full feature set from npm-check-updates

# Notes

- nrush is short for npm rush, as in "please get me up to date, I want the bleeding edge stuff and I love suffering"
- this completely ignores version ranges set in your `package.json`. Watch out.
- couldn't get this to build on Mac. I'll fix it eventually.
