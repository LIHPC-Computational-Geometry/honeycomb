# Generate docs

cargo +nightly doc --all --no-deps --all-features
mdbook build user-guide/

# Add stuff to the deployed folder

mv user-guide/book target/doc/user-guide
cp docs/index.html target/doc
cp docs/index.css target/doc
