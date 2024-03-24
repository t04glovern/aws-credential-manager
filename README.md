# aws-credential-manager

A demo Tauri + React + Typescript app!

![Demo](./docs/demo.png)

## Recommended IDE Setup

- Ideally, run this in a devcontainer.
- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Generate Icons

```bash
# Install dependencies
pip3 install pillow==10.2.0 icnsutil==1.1.0

# Ensure you have a high resolution icon.png
./iconize.py --icon src-tauri/icons/icon.png --output src-tauri/icons
```

## Development

> **NOTE**: It is highly recommend, and more or less required for you to run `wslg` X Server on Windows: [https://github.com/microsoft/wslg](https://github.com/microsoft/wslg). This repository is setup to use `export DISPLAY=:0` so that you can view the app when working in a devcontainer.

Press `F5` to start the app in development mode.

Alternatively, run the following commands:

```bash
npm run tauri dev
```

If you want to develop the React app separately, run the following commands:

```bash
npm run dev
# open http://localhost:1420/
```

## Build

```bash
npm run tauri build
```
