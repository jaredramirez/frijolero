# Frijolero

> A platformer game written in Rust using Bevy

## Running it

We use `devenv` to install everything needed to run the game. 

### Installing Devenv

> This section is copied from the [devenv getting started guide][devenv-start].
> Please refer to it for the most up-to-date instructions and additional
> installation options.

To install devenv, you first have to install [Nix][nix]:
```sh
# MacOS
curl -L https://raw.githubusercontent.com/NixOS/experimental-nix-installer/main/nix-installer.sh | sh -s install

# Linux
sh <(curl -L https://nixos.org/nix/install) --daemon
```

Now, we can install devenv:
```sh
nix-env --install --attr devenv -f https://github.com/NixOS/nixpkgs/tarball/nixpkgs-unstable
```

Now, make sure `devenv` is available on your `PATH`:
```sh
devenv --version
```

Should print something like:

```
devenv 1.3.1 (aarch64-darwin)
```

[devenv-start]: https://devenv.sh/getting-started/#__tabbed_3_1
[nix]: https://nixos.org/

### Using devenv to install everything else

Okay, now that we have `devenv` installed, we can use it to install _everything_
else (except direnv). To spin up a dev shell with everything installed, go to
the project directory and run:

```sh
devenv shell
```

At this point, you should see a lot of thing happening. This is `devenv`
installing everything needed to build game. Once it's done, you should see
something like:

```
• Building shell ...
• Using Cachix: devenv
✔ Building shell in 0.1s.
• Entering shell
Running tasks     devenv:enterShell
Succeeded         devenv:enterShell        7ms
2 Succeeded                                25.19ms

(devenv) bash-5.2$
```

Now you're inside the devshell!

### Running the game

To run the game, run the following command inside the devshell:

```sh
just run
```

You should see other things installing now, this is the tool `cargo` installing
everything needed to run the game. This will likely take several minutes. Once
it's done, a window with the game running should appear!

### Choosing which level to load

Now, by default running the game loads the level in
`assets/levels/test/level.ldtk`. To change which level to load, run:

```sh
just run level="path/to/my/level.ldtk"
```
