. scripts/lib/common.bash
. scripts/lib/client.bash

echo "Starting client"
(
    cd $CLIENT_DIR
    ng serve --open
)