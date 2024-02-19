# honeycomb-guide

---

**honeycomb-guide** is the mdbook project used to generate the documentation you are
currently reading. Its content mainly focuses on definition and feature-listing 
rather than technical details on implementation. The latter can be found in the code 
documentation.

## Building


You can generate this documentation locally using **mdbook** and **cargo doc**:

```shell
# Serve the doc on a local server
mdbook serve --open -d ../target/doc/ honeycomb-guide/ &
cargo doc --all --no-deps

# Kill the local server
kill $(pidof mdbook) 

# Without pidof
kill $(ps -e | awk '/mdbook/ {print $1}')
```

Note that if you edit the user guide's content, you will have to generate the rust doc 
again as mdbook remove all files of the target folder at each update.

## Contributing

A few observations on writing documentation using **mdbook**:

- When linking to a folder containing an `index.html` file, be sure to include the last 
  `/` in the name of the folder if you don't name the index file directly. Not including 
  that last character seems to break in-file linking of the local version.