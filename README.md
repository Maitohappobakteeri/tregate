# tregate


## New rust lib

https://www.rust-lang.org/tools/install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

https://developer.mozilla.org/en-US/docs/WebAssembly/Rust_to_wasm
cargo install wasm-pack
cargo new --lib hello-wasm

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

wasm-pack build --target web



wasm-pack build --target bundler
npm install ../hello-wasm/pkg

## New angular client

https://github.com/nvm-sh/nvm#install--update-script
nvm install node
nvm use node
sudo npm install -g @angular/cli
ng new client
cd client
ng serve 
