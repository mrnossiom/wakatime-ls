# WakaTime Language Server

<p align="center"><strong>
A dead-simple language server around <code>wakatime-cli</code> to send code tracking heartbeats
</strong></p>

<p align="center">
  <a href="https://nixos.org/">
    <img alt="Nix Powered" src="https://img.shields.io/badge/Nix-Powered-blue?logo=nixos" />
  </a>
  <a href="https://matrix.to/#/#wakatime-ls:wiro.world">
    <img alt="Matrix room at #wakatime-ls:wiro.world" src="https://img.shields.io/badge/Matrix-%23wakatime--ls%3Awiro.world-white?logo=matrix">
  </a>
</p>

I made this language server wrapper implementation around `wakatime-cli` because I wanted support for WakaTime in [Helix](https://github.com/helix-editor/helix). That said, it's should be compatible with every LSP implementation.

`wakatime-ls` listens to open, change, save and close file events. It then reflects this activity by sending heartbeats via the `wakatime-cli`.

# Installation

<details>
  <summary>With <code>cargo</code></summary><br />

Install from repository with cargo:

```sh
# from crates.io
cargo install wakatime-ls
# or from source
cargo install --git https://github.com/mrnossiom/wakatime-ls
```

</details>

<details>
  <summary>With <code>nix</code> flakes</summary><br />

A `flake.nix` is available which means that you can use `github:mrnossiom/wakatime-ls` as a flake identifier. Package is reachable through `packages.${system}.default` or `packages.${system}.wakatime-ls`. That way you can:

- import this repository in your flake inputs

  ```nix
  {
    wakatime-ls.url = "github:mrnossiom/wakatime-ls";
    wakatime-ls.inputs.nixpkgs.follows = "nixpkgs";
  }
  ```

  Add the package to your [NixOS](https://nixos.org/) or [Home Manager](https://github.com/nix-community/home-manager) packages depending on your installation.

- use with `nix shell` for temporary testing

  e.g. `nix shell github:mrnossiom/wakatime-ls`

- use with `nix profile` for imperative installation

  e.g. `nix profile install github:mrnossiom/wakatime-ls`

</details>

<details>
  <summary>Download binary from GitHub releases</summary><br />

Find the latest `wakatime-ls` release on GitHub [here](https://github.com/mrnossiom/wakatime-ls/releases).

You may download the compressed tarball corresponding to your OS.

</details>

You will also need `wakatime-cli` in your `$PATH` for setups others than Nix, which already bundles it. You can download `wakatime-cli`:

- with your preferred package manager (see [`wakatime-cli` repology])
- from the [`wakatime-cli` releases page]

[`wakatime-cli` repology]: https://repology.org/project/wakatime-cli/versions
[`wakatime-cli` releases page]: https://github.com/wakatime/wakatime-cli/releases/latest

# Configuration

Currently `wakatime-ls` is not configurable cause it's more of a simple `wakatime-cli` wrapper which itself is configurable with `$WAKATIME_HOME/.wakatime.cfg` ([see docs](https://github.com/wakatime/wakatime-cli/blob/develop/USAGE.md#ini-config-file)). (`$WAKATIME_HOME` defaults to `$HOME`)

It might be already filled if you've used another WakaTime plugin in the past.

Required configuration is to set your [WakaTime api key] in `$WAKATIME_HOME/.wakatime.cfg`, like so:

[WakaTime api key]: https://wakatime.com/settings/api-key

```ini
[settings]
api_key=waka_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

# Usage

## Helix

You *need* to add `wakatime-ls` to your `languages.toml` configuration ([see docs](https://docs.helix-editor.com/languages.html)). Though, it's not currently possible to add global language servers, you can add `wakatime` for your most significant languages.

Helix does not merge values, you also need to specify the original language servers defined for the language. You can find them in the default [`languages.toml`](https://github.com/helix-editor/helix/blob/master/languages.toml)


Adding global language servers is blocking on [Helix's new config system](https://github.com/helix-editor/helix/pull/9318).

e.g.

```toml
[language-server.wakatime]
command = "wakatime-ls"

[[language]]
name = "markdown"
language-servers = ["marksman", "wakatime"]

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "wakatime"]

[[language]]
name = "nix"
language-servers = ["nil", "wakatime"]

# add as many entries as the number of the languages you use
```

## And more...

`wakatime-ls` is a language server, it's easy to quickly integrate into any editor that support the language server protocol.

Feel free to make a PR for your own favorite editor.

# License

Work is licensed under [CECILL-2.1](https://choosealicense.com/licenses/cecill-2.1/), a French OSS license that allows modification and distribution of the software while requiring the same license for derived works.
