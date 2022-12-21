# Presentation Language Tool

A simple tool to parse the TLS/MLS presentation language for research and learning purposes.

# Usage

Currently, this tool generates a dependency graph of all `structs` and `enums` defined in a given `<file>` and output the graph in either [GML] or [GV] format.

```sh
plt <file> [gml|gv]'
```

You can try this out by running ...

```sh
cargo run -- assets/draft-ietf-mls-protocol.tls
```

... and using a GML viewer such as [yEd]. (Don't forget to click on auto-layout.)

Note: This tool is not really tested and might produce invalid GML/GV output. Use at your own risk!

## License

Licensed under either of ...

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

... at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[GML]: https://en.wikipedia.org/wiki/Graph_Modelling_Language
[GV]: https://graphviz.org/doc/info/lang.html
[yEd]: https://www.yworks.com/products/yed
