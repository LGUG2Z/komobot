# komobot

The Komorebi Community Discord Server bot.

<p>
  <a href="https://techforpalestine.org/learn-more">
    <img alt="Tech for Palestine" src="https://badge.techforpalestine.org/default">
  </a>
  <a href="https://discord.gg/mGkn66PHkx">
    <img alt="Discord" src="https://img.shields.io/discord/898554690126630914">
  </a>
  <a href="https://github.com/sponsors/LGUG2Z">
    <img alt="GitHub Sponsors" src="https://img.shields.io/github/sponsors/LGUG2Z">
  </a>
  <a href="https://ko-fi.com/lgug2z">
    <img alt="Ko-fi" src="https://img.shields.io/badge/kofi-tip-green">
  </a>
  <a href="https://notado.app/feeds/jado/software-development">
    <img alt="Notado Feed" src="https://img.shields.io/badge/Notado-Subscribe-informational">
  </a>
  <a href="https://www.youtube.com/channel/UCeai3-do-9O4MNy9_xjO6mg?sub_confirmation=1">
    <img alt="YouTube" src="https://img.shields.io/youtube/channel/subscribers/UCeai3-do-9O4MNy9_xjO6mg">
  </a>
</p>

_komobot_ helps users on the [Komorebi Community Discord Server](https://discord.gg/mGkn66PHkx) who
support [komorebi](https://github.com/LGUG2Z/komorebi) via different platforms to self-assign Discord reflecting their
contributions.

# Commands

- `/github-sponsor [username]`
  - This command is used to self-assign one of the GitHub Sponsors Tier roles on the Komorebi Community Discord Server

# Contribution Guidelines

If you would like to contribute to `komobot` please take the time to carefully read the guidelines below.

## Commit hygiene

- Flatten all `use` statements
- Run `cargo +stable clippy` and ensure that all lints and suggestions have been addressed before committing
- Run `cargo +nightly fmt --all` to ensure consistent formatting before committing
- Use `git cz` with
  the [Commitizen CLI](https://github.com/commitizen/cz-cli#conventional-commit-messages-as-a-global-utility) to prepare
  commit messages
- Provide **at least** one short sentence or paragraph in your commit message body to describe your thought process for the
  changes being committed

## License

`komobot` is licensed under the [Komorebi 1.0.0 license](./LICENSE.md), which
is a fork of the [PolyForm Strict 1.0.0
license](https://polyformproject.org/licenses/strict/1.0.0). On a high level
this means that you are free to do whatever you want with `komobot` for
personal use other than redistribution, or distribution of new works (i.e.
hard-forks) based on the software.

Anyone is free to make their own fork of `komobot` with changes intended
either for personal use or for integration back upstream via pull requests.

_The [Komorebi 1.0.0 License](./LICENSE.md) does not permit any kind of
commercial use._

### Contribution licensing

Contributions are accepted with the following understanding:

- Contributed content is licensed under the terms of the 0-BSD license
- Contributors accept the terms of the project license at the time of contribution

By making a contribution, you accept both the current project license terms, and that all contributions that you have
made are provided under the terms of the 0-BSD license.

#### Zero-Clause BSD

```
Permission to use, copy, modify, and/or distribute this software for
any purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL
WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE
FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```
