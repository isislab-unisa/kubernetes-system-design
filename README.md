## Book for Kubernetes System Design

This book originated as an introduction to Kubernetes system design for Unisa students, but it is freely available for anyone interested in learning how to design Kubernetes architectures.

## Prerequisites

- [prek](https://github.com/j178/prek?tab=readme-ov-file) (pre-commit hook manager)

- [excalidraw](https://excalidraw.com/) (diagramming tool)
    - use VSCode extension for excalidraw to edit diagrams in VSCode [link](https://marketplace.visualstudio.com/items?itemName=pomdtr.excalidraw-editor)
    - download excalidraw assets from [assets/README.md](assets/README.md) and add them to your excalidraw library for use in diagrams.
    - after downloading assets, you can import them into excalidraw by going to the library tab and importing the downloaded .excalidrawlib files.

- [mdbook](https://rust-lang.github.io/mdBook/guide/installation.html)
    - install mdbook using cargo: `cargo install mdbook`
    - run the book locally: `mdbook serve`
    - run then book locally with live reload: `mdbook serve --watcher native`
    - build the book for deployment: `mdbook build`
    - mdbook will use preprocessor contained in the `preprocessor.subsection-numbering` section of `book.toml` to automatically number sections and subsections. The preprocessor is implemented in Rust and can be found in the `enumeration` directory. To build the preprocessor, run `cargo build --release --manifest-path=../enumeration/Cargo.toml --locked`. The preprocessor will be automatically run by mdbook when building the book.

## Playground

You can access a Kubernetes playground where you can experiment with tasks at [killercoda](https://killercoda.com/isislab/scenario/exam-playground).

## Contributing

You can contribute to this project by submitting pull requests on GitHub. We welcome contributions of all kinds, including bug fixes, improvements to the content, and suggestions for new exercises or topics to cover.
You can find more detailed instructions on how to contribute in the [CONTRIBUTING.md](CONTRIBUTING.md) file.
