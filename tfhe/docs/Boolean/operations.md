# Operations

In tfhe::boolean, the available operations are mainly related to their equivalent Boolean gates (i.e., AND, OR... etc). What follows is an example of a unary gate (NOT) and one about a binary gate (XOR). The last one is about the ternary MUX gate, which gives the possibility to homomorphically compute conditional statements of the form `If..Then..Else`.

## The NOT unary gate

```rust
use tfhe::boolean::prelude::*;

fn main() {
// We generate a set of client/server keys, using the default parameters:
    let (client_key, server_key) = gen_keys();

// We use the client secret key to encrypt a message:
    let ct_1 = client_key.encrypt(true);
    
// We use the server public key to execute the NOT gate:
    let ct_not = server_key.not(&ct_1);

// We use the client key to decrypt the output of the circuit:
    let output = client_key.decrypt(&ct_not);
    assert_eq!(output, false);
}
```

## Binary gates

```rust
use tfhe::boolean::prelude::*;

fn main() {
// We generate a set of client/server keys, using the default parameters:
    let (mut client_key, mut server_key) = gen_keys();

// We use the client secret key to encrypt a message:
    let ct_1 = client_key.encrypt(true);
    let ct_2 = client_key.encrypt(false);
    
// We use the server public key to execute the XOR gate:
    let ct_xor = server_key.xor(&ct_1, &ct_2);

// We use the client key to decrypt the output of the circuit:
    let output = client_key.decrypt(&ct_xor);
    assert_eq!(output, true^false);
}
```

## The MUX ternary gate

Let `ct_1, ct_2, ct_3` be three Boolean ciphertexts. Then, the MUX gate (abbreviation of MUtipleXer) is equivalent to the operation:

```r
if ct_1 {  
    return ct_2
} else {
    return ct_3
}
```

This example shows how to use the MUX ternary gate:

```rust
use tfhe::boolean::prelude::*;

fn main() {
// We generate a set of client/server keys, using the default parameters:
    let (mut client_key, mut server_key) = gen_keys();

    let bool1 = true;
    let bool2 = false;
    let bool3 = true;
    
// We use the client secret key to encrypt a message:
    let ct_1 = client_key.encrypt(true);
    let ct_2 = client_key.encrypt(false);
    let ct_3 = client_key.encrypt(false);


// We use the server public key to execute the NOT gate:
    let ct_xor = server_key.mux(&ct_1, &ct_2, &ct_3);

// We use the client key to decrypt the output of the circuit:
    let output = client_key.decrypt(&ct_xor);
    assert_eq!(output, if bool1 {bool2} else {bool3});
}
```
