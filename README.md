<div align="center">
  credit-store-demo (2025)
  <p>A demo for an event sourcing implementation in Rust with diesel (WIP)</p>
</div>

<p align="center">
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-brightgreen.svg"
      alt="License: MIT" />
  </a>
  <a href="https://buymeacoffee.com/lan22h">
    <img src="https://img.shields.io/static/v1?label=Buy me a coffee&message=%E2%9D%A4&logo=BuyMeACoffee&link=&color=greygreen"
      alt="Buy me a Coffee" />
  </a>
</p>

<div align="center">
  <sub>Built with ❤︎ by Mohammed Alzakariya
</div>
<br>

# Purpose

We would like our database tables to preserve history in an append-only event store, and to have the ability to version them and seamlessly switch between versions or undo/redo actions.

# Related

You can find associated notes for this demo at delta-trace [here](https://github.com/deltatraced/delta-trace/tree/webview/lan/protos/2025/001%20Rust%20Diesel%20Event%20Sourcing).

# Setup

After diesel CLI is run, we may need to do some postprocessing for enums support. Use the tool `diesel-postprocess.py` in scripts.

Install requirements:

```sh
python3 -m pip install -r requirements.txt
```

For full cycle regeneration,

```sh
source ./.env && rm $DATABASE_URL; diesel migration run && python3 scripts/diesel-postprocess.py
```

# Contributing

All contributions are welcome!

Just open an issue or a PR for any suggestions, questions, problems, or additions.

You can reach out to me via mailto:lanhikarixx@gmail.com.

# License

This work is licensed under the [MIT license](https://opensource.org/licenses/mit-license.php) © 2025 Mohammed Alzakariya.
