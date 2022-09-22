MAP_LOADER_DIR=map-loader/

map_loader_build(){
    echo "Building map loader"
    cd $MAP_LOADER_DIR
    wasm-pack build --target bundler
}