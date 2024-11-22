# Hyprkube
## Introduction
Managing large fleets of Kubernetes clusters can be challenging and repetitive. Hyprkube aims to
make a cluster administrator's life easier by rethinking how we interact with resources in Kubernetes.

Please note that this project is still far away from an MVP-status and is not suitable for production use.

## Highlights
- **Customizable resource views** allow you to build resource tables that show exactly what you want to see - powered
by [Rhai](https://rhai.rs/) scripts.
- **Pinned Kinds** allow you to focus on the resources you care about - no more scrolling through
an endless list of CRDs

## Development
Hyprkube is built on [Tauri](https://tauri.app) and [React](https://react.dev).
Follow the official [Tauri docs](https://tauri.app/start/prerequisites/) to learn how to setup the
development environment, including installing Node.js and Rust.

## Building
Hyprkube can be built by running `npm run tauri build`. The build process currently produces deb and rpm packages.
Other platform might follow at some point.
