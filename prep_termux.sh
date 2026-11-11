cargo vendor
find ./vendor \
-mindepth 1 -maxdepth 1 -type d \
! -wholename ./vendor/rustls-platform-verifier \
-exec rm -rf '{}' \;

find vendor/rustls-platform-verifier -type f -print0 | \
xargs -0 sed -i \
-e 's|"android"|"disabling_this_because_it_is_for_building_an_apk"|g' \
-e "s|ANDROID|DISABLING_THIS_BECAUSE_IT_IS_FOR_BUILDING_AN_APK|g" \
-e 's|"linux"|"android"|g'

echo "" >> Cargo.toml
echo '[patch.crates-io]' >> Cargo.toml
echo 'rustls-platform-verifier = { path = "./vendor/rustls-platform-verifier" }' >> Cargo.toml
