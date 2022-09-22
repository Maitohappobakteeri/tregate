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

vscode = rust-analyzer
auto closing angle bracket enable
turn on format on save in vscode

"editor.formatOnSave": true,
"editor.formatOnType": true,
"rust-analyzer.rustfmt.enableRangeFormatting": true,
"[rust]": {
"editor.defaultFormatter": "matklad.rust-analyzer",
"editor.formatOnSave": true
},

curl 'https://beta-karttakuva.maanmittauslaitos.fi/ortokuvat-ja-korkeusmallit/wcs/v1?service=WCS&version=2.0.1&request=GetCoverage&CoverageID=korkeusmalli_2m&SUBSET=E(326874,329810)&SUBSET=N(6819455,6824888)&format=text/plain' > map-tool/data/heightgrid.txt

## New angular client

https://github.com/nvm-sh/nvm#install--update-script
nvm install node
nvm use node
sudo npm install -g @angular/cli
ng new client
cd client
ng serve

npm install --save-dev @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint
npm install eslint-plugin-sonarjs --save-dev

vscode prettier
