# Presentation Language Tool

A simple tool to parse the TLS/MLS presentation language for research and learning purposes.

# Usage

Currently, this tool generates a dependency graph of all `structs` and `enums` defined in a given `<file>` and outputs a graph in either [GML] or [GV] format.

```sh
plt <file> [gml|gv]
```

You can try this out by running ...

```sh
cargo run -- assets/draft-ietf-mls-protocol.tls
```

... and using a GML viewer such as [yEd]. (Don't forget to apply an auto-layout.)

> [!WARNING]
> This tool is not thoroughly tested and might produce invalid GML/GV output. Use at your own risk!

# Example

The following definition ...

```c
struct {    
    CredentialType credential_type;    
    select (Credential.credential_type) {    
        case basic:    
            opaque identity<V>;    
    
        case x509:    
            Certificate chain<V>;    
    };    
} Credential;
```

... is parsed into ...

```rust
Struct(
    Struct {
        name: "Credential",
        items: [
            Field(
                Field {
                    type: "CredentialType",
                    name: "credential_type",
                    range: None,
                    optional: false,
                    default: None,
                },
            ),
            Select(
                Select {
                    over: "Credential.credential_type",
                    cases: [
                        Case {
                            left: "basic",
                            right: Fields(
                                [
                                    Field {
                                        type: "opaque",
                                        name: "identity",
                                        range: Some(
                                            Variable,
                                        ),
                                        optional: false,
                                        default: None,
                                    },
                                ],
                            ),
                        },
// ...
                    ],
                },
            ),
        ],
    },
)
```

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
