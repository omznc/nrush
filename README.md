<center>

![nrush](https://github.com/omznc/nrush/assets/38432561/ad2f9d0c-477a-420a-aa34-7c171fe8a0a8)

</center>

<div style="display: flex;">
    <a href="https://www.npmjs.com/package/nrush">
        <img src="https://img.shields.io/npm/dw/nrush?style=for-the-badge&logo=npm" alt="npm">
    </a>
    <img src="https://img.shields.io/github/actions/workflow/status/omznc/nrush/cd.yml?style=for-the-badge&logo=github" alt="GitHub">
</div>





# nrush 🦀 

> A speedy way to update all packages in a Node/Bun project. (probably) less featured, less tested, and less
> reliable alternative to npm-check-updates, but hey, it's blazingly fast 🔥



## Usage

Bun

```
bun add -g nrush@latest
nrush
```

Node

```
npm i -g nrush@latest
nrush
```

**Commands:**

- `nrush about` - Display comprehensive information about NRush.
- `nrush help` - Provide a usage guide for NRush. Primarily, this section.

**Arguments:**
(Arguments are applicable only if no commands are supplied and only `nrush` is executed.)

1. Update Options (`-u` / `--update`):
	- Automatically updates all dependencies without user interaction.

2. Interactive Mode (`-i` / `--interactive`)
	- User can select which packages to update. Defaults to this if both `-u` and `-i` are supplied.

3. Include (`--include <dev,peer>`):
	- Include `dev` and/or `peer` dependencies in the update process.

4. Path Specification (`-p <path>` / `--path <path>`):
	- Specify the path to a `package.json` file. The default is the current directory.

5. Skip Ranges in Versioning (`--skip-ranges`):
	- Skips version ranges in package.json. Default is `false`, preserving them.
	- Example: `^1.0.0` will be updated to `2.0.0` if `--skip-ranges` is supplied.

6. Update Any Version (`--update-any`):
	- Updates `*` versions in package.json. Default is `false`, maintaining them.
	- Example: `*` will be updated to `2.0.0` if `--update-any` is supplied.

7. Semver Constraint (`-s <semver>` / `--semver <semver>`):
	- Specify a maximum semver range to update to. Choose either `major`, `minor`, or `patch`. Default is `major`.
    - This currently does nothing.

By executing `nrush` without any arguments or commands, a list of updatable packages will be displayed, and you'll be prompted to install them.

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
