# shapefile-utils

This project enables Rustaceans to *read* [Shapefiles](http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf). These are basically files which can be generated from geographic data, and contain geographic features (in a SHP file) alongside with metadata (in a DBF file).

# How to use this stuff

## Adding the dependency

Currently, this crate is not on crates.io, since it is not yet considered stable and the API will probably undergo some more changes. Therefore you will have to add this project as [a git dependency](http://doc.crates.io/specifying-dependencies.html#specifying-dependencies-from-git-repositories) in your Cargo.toml file.

## Starting to code

Add this to one of your files:

```rust
extern crate shapefile_utils;
```

Then you can start using it. The first step is to insert some `use` clauses.

```rust
// The following use clauses are needed
use std::path::Path;
use shapefile_utils::Shapefile;
```

Then you can create a Shapefile object, for which you need to specify, in this order, the SHP file, the SHX file, and the DBF file paths.

```rust
let mut my_shapefile = Shapefile::new(
    &Path::new("assets/test.shp"), 
    &Path::new("assets/test.shx"), 
    &Path::new("assets/test.dbf")).unwrap();
```

Now you can iterate over the entries in the Shapefile, like so:

```rust
for record in my_shapefile {
    println!("Something called {:?}", record.metadata.get(&String::from("name")).unwrap());
}
```

This will print a list with all object names in the Shapefile (if there is a column in the DBF file that is called `name`, at least).

You can try all of this with the test files which are part of this repository (see `assets/`).

Good luck!

# Missing features

* Moar unit tests & robustness tests
* Restrict object visibility as much as possible
* Review Shape parsing
* Moar documentation comments
* ???
* Writing, if anybody has a use case for that

# License

This project has the MIT license.
