# Small Local Ai UI
This is a simple way to interact with local llms.

## Setup using the .exe
This should be straight forward.
- Download the x64-setup.exe from release v0.1.0
- Run the setup exe
- Download a gguf model from huggingface
    - For example googles gemma 4B model
    - https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF
- Enter the path of the downloded model into the program

## Setup using Tauri
This is for people wanting to compile the program from source code, and be able to edit and make local changes.
- Follow the tauri setup from the tauri site
    - https://tauri.app/start/
    - There should be a existing rust setup using cargo
- Get the create-tauri-app crate
    - cargo install create-tauri-app --locked
- Get the tauri cli
    - cargo install tauri-cli --version "^2.0.0" --locked
- Get libclang as this is needed for llama-cpp
    - For linux: sudo pacman -S clang
    - For windows: winget install LLVM.LLVM
- Compile the program using the build command
    - cargo tauri build
- The executable program is found in the folder src-tauri/target/release/
- - Download a gguf model from huggingface
    - For example googles gemma 4B model
    - https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF
- Enter the path of the downloded model into the program
