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
- **Extensions** allow you to extend the functionality of Hyprkube in a safe and controlled manner.

## Development
Hyprkube is built on [Tauri](https://tauri.app) and [React](https://react.dev).
Follow the official [Tauri docs](https://tauri.app/start/prerequisites/) to learn how to setup the
development environment, including installing Node.js and Rust.

## Building
Hyprkube can be built by running `npm run tauri build`. The build process currently produces deb and rpm packages.
Other platform might follow at some point.

## Extensions
Hyprkube can be extended to integrate with your daily workflows by creating extensions. These extensions can be used to
extend the context menus of Kubernetes resources with custom actions. Extensions can be placed in:

- Linux: `~/.local/share/de.webd97.hyprkube/extensions`
- macOS: `~/Library/Application Support/de.webd97.hyprkube/extensions`
- Windows: `%APPDATA%\Roaming\de.webd97.hyprkube\extensions`

Each extension is a directory with the following optional subdirectories - depending on the functionality that you want
to add. The capabilities of each script are determined by the subdirectory they live in. At runtime, these are mapped to
specialized scripting engines with exactly the features that are permitted. This ensures both fast execution and also
prevents undesirable side effects if a scripts tries to do things that it should not do.

```
~/.local/share/de.webd97.hyprkube/extensions
├── cert-manager
│   └── menus
│       ├── 000-my-shortcuts.rhai
│       └── 100-extras.rhai
└── fleet
    └── menus
        └── 000-gitrepo.rhai
```

### Custom resource context menus
Scripts in the `menus` subdirectory can be used to extend the context menus of Kubernetes resources. The following
Rhai modules are available:

- `base64`
- `kube`
- `frontend`
- `clipboard`
