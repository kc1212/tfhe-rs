<p align="center">
<!-- product name logo -->
  <img width=600 src="https://user-images.githubusercontent.com/86411313/201107820-b1b861be-6b3f-46cc-bccd-ed051201781a.png">
</p>
<p align="center">
<!-- Version badge using shields.io -->
  <a href="https://github.com/zama-ai/tfhe-rs/releases">
    <img src="https://img.shields.io/github/v/release/zama-ai/tfhe-rs?style=flat-square">
  </a>
<!-- Link to docs badge using shields.io -->
  <a href="https://docs.zama.ai/tfhe-rs">
    <img src="https://img.shields.io/badge/read-documentation-yellow?style=flat-square">
  </a>
<!-- Community forum badge using shields.io -->
  <a href="https://community.zama.ai">
    <img src="https://img.shields.io/badge/community%20forum-online-brightgreen?style=flat-square">
  </a>
<!-- Open source badge using shields.io -->
  <a href="https://docs.zama.ai/tfhe-rs/developers/contributing">
    <img src="https://img.shields.io/badge/we're%20open%20source-contributing.md-blue?style=flat-square">
  </a>
<!-- Follow on twitter badge using shields.io -->
  <a href="https://twitter.com/zama_fhe">
    <img src="https://img.shields.io/badge/follow-zama_fhe-blue?logo=twitter&style=flat-square">
  </a>
</p>

**TFHE-rs** is a pure Rust implementation of TFHE for boolean and small integer
arithmetics over encrypted data. It includes:
 - a **Rust** API
 - a **C** API
 - and a **client-side WASM** API

**TFHE-rs** is meant for developers and researchers who want full control over
what they can do with TFHE, while not having to worry about the low level
implementation. The goal is to have a stable, simple, high-performance, and
production-ready library for all the advanced features of TFHE.

## Getting Started

To use the latest version of `TFHE-rs` in your project, you first need to add it as a dependency in your `Cargo.toml`:

+ For x86_64-based machines running Unix-like OSes:

```toml
tfhe = { version = "*", features = ["boolean", "shortint", "integer", "x86_64-unix"] }
```

+ For Apple Silicon or aarch64-based machines running Unix-like OSes:

```toml
tfhe = { version = "*", features = ["boolean", "shortint", "integer", "aarch64-unix"] }
```
Note: users with ARM devices must use `TFHE-rs` by compiling using the `nightly` toolchain.


+ For x86_64-based machines with the [`rdseed instruction`](https://en.wikipedia.org/wiki/RDRAND) 
running Windows:

```toml
tfhe = { version = "*", features = ["boolean", "shortint", "integer", "x86_64"] }
```

Note: aarch64-based machines are not yet supported for Windows as it's currently missing an entropy source to be able to seed the [CSPRNGs](https://en.wikipedia.org/wiki/Cryptographically_secure_pseudorandom_number_generator) used in TFHE-rs

Here is a full example evaluating a Boolean circuit:

```rust
use tfhe::boolean::prelude::*;

fn main() {
// We generate a set of client/server keys, using the default parameters:
    let (mut client_key, mut server_key) = gen_keys();

// We use the client secret key to encrypt two messages:
    let ct_1 = client_key.encrypt(true);
    let ct_2 = client_key.encrypt(false);

// We use the server public key to execute a boolean circuit:
// if ((NOT ct_2) NAND (ct_1 AND ct_2)) then (NOT ct_2) else (ct_1 AND ct_2)
    let ct_3 = server_key.not(&ct_2);
    let ct_4 = server_key.and(&ct_1, &ct_2);
    let ct_5 = server_key.nand(&ct_3, &ct_4);
    let ct_6 = server_key.mux(&ct_5, &ct_3, &ct_4);

// We use the client key to decrypt the output of the circuit:
    let output = client_key.decrypt(&ct_6);
    assert_eq!(output, true);
}
```

Another example of how the library can be used with shortints:

```rust
use tfhe::shortint::prelude::*;

fn main() {
    // Generate a set of client/server keys, using the default parameters:
    let (client_key, server_key) = gen_keys(Parameters::default());

    let msg1 = 3;
    let msg2 = 2;

    // Encrypt two messages using the (private) client key:
    let ct_1 = client_key.encrypt(msg1);
    let ct_2 = client_key.encrypt(msg2);

    // Homomorphically compute an addition
    let ct_add = server_key.unchecked_add(&ct_1, &ct_2);

    // Define the Hamming weight function
    // f: x -> sum of the bits of x
    let f = |x:u64| x.count_ones() as u64;

    // Generate the accumulator for the function
    let acc = server_key.generate_accumulator(f);

    // Compute the function over the ciphertext using the PBS
    let ct_res = server_key.keyswitch_programmable_bootstrap(&ct_add, &acc);

    // Decrypt the ciphertext using the (private) client key
    let output = client_key.decrypt(&ct_res);
    assert_eq!(output, f(msg1 + msg2));
}
```

## Contributing

There are two ways to contribute to TFHE-rs:

- you can open issues to report bugs or typos, or to suggest new ideas
- you can ask to become an official contributor by emailing [hello@zama.ai](mailto:hello@zama.ai).
(becoming an approved contributor involves signing our Contributor License Agreement (CLA))

Only approved contributors can send pull requests, so please make sure to get in touch before you do!

## Credits

This library uses several dependencies and we would like to thank the contributors of those
libraries.

## License

This software is distributed under the BSD-3-Clause-Clear license. If you have any questions,
please contact us at `hello@zama.ai`.

## Disclaimers

### Security Estimation

Security estimations are done using the
[Lattice Estimator](https://github.com/malb/lattice-estimator)
with `red_cost_model = reduction.RC.BDGL16`.

When a new update is published in the Lattice Estimator, we update parameters accordingly.

### Side-Channel Attacks

Mitigation for side channel attacks have not yet been implemented in TFHE-rs,
and will be released in upcoming versions.
