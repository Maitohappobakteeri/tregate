. scripts/lib/common.bash

echo "Building map"
(
    cd map-tool/
    cargo run --release
)

mkdir -p client/src/assets/generated/
cp map-tool/output/map.json client/src/assets/generated/map.json
cp map-tool/output/height_model.json client/src/assets/generated/height_model.json
cp map-tool/output/height_normals.json client/src/assets/generated/height_normals.json
cp map-tool/output/building_models.json client/src/assets/generated/building_models.json
cp map-tool/output/building_normals.json client/src/assets/generated/building_normals.json
