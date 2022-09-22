CLIENT_DIR=client/

client_init() {
    (
        echo "Installing npm packages"
        cd $CLIENT_DIR
        npm install
    )
}