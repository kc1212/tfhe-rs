use crate::integer::ciphertext::RadixCiphertext;
use crate::integer::server_key::CheckError;
use crate::integer::server_key::CheckError::CarryFull;
use crate::integer::ServerKey;

impl ServerKey {
    /// Computes homomorphically an addition between two ciphertexts encrypting integer values.
    ///
    /// This function computes the operation without checking if it exceeds the capacity of the
    /// ciphertext.
    ///
    /// The result is returned as a new ciphertext.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 10;
    /// let msg2 = 127;
    ///
    /// let ct1 = cks.encrypt(msg1);
    /// let ct2 = cks.encrypt(msg2);
    ///
    /// // Compute homomorphically an addition:
    /// let ct_res = sks.unchecked_add(&ct1, &ct2);
    ///
    /// // Decrypt:
    /// let dec_result = cks.decrypt(&ct_res);
    /// assert_eq!(dec_result, msg1 + msg2);
    /// ```
    pub fn unchecked_add(
        &self,
        ct_left: &RadixCiphertext,
        ct_right: &RadixCiphertext,
    ) -> RadixCiphertext {
        let mut result = ct_left.clone();
        self.unchecked_add_assign(&mut result, ct_right);
        result
    }

    /// Computes homomorphically an addition between two ciphertexts encrypting integer values.
    ///
    /// This function computes the operation without checking if it exceeds the capacity of the
    /// ciphertext.
    ///
    /// The result is assigned to the `ct_left` ciphertext.
    /// ```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 28;
    /// let msg2 = 127;
    ///
    /// let mut ct1 = cks.encrypt(msg1);
    /// let ct2 = cks.encrypt(msg2);
    ///
    /// // Compute homomorphically an addition:
    /// sks.unchecked_add_assign(&mut ct1, &ct2);
    ///
    /// // Decrypt:
    /// let dec_ct1 = cks.decrypt(&ct1);
    /// assert_eq!(dec_ct1, msg1 + msg2);
    /// ```
    pub fn unchecked_add_assign(&self, ct_left: &mut RadixCiphertext, ct_right: &RadixCiphertext) {
        for (ct_left_i, ct_right_i) in ct_left.blocks.iter_mut().zip(ct_right.blocks.iter()) {
            self.key.unchecked_add_assign(ct_left_i, ct_right_i);
        }
    }

    /// Verifies if ct1 and ct2 can be added together.
    ///
    /// # Example
    ///
    ///```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 46;
    /// let msg2 = 87;
    ///
    /// let ct1 = cks.encrypt(msg1);
    /// let ct2 = cks.encrypt(msg2);
    ///
    /// // Check if we can perform an addition
    /// let res = sks.is_add_possible(&ct1, &ct2);
    ///
    /// assert_eq!(true, res);
    /// ```
    pub fn is_add_possible(&self, ct_left: &RadixCiphertext, ct_right: &RadixCiphertext) -> bool {
        for (ct_left_i, ct_right_i) in ct_left.blocks.iter().zip(ct_right.blocks.iter()) {
            if !self.key.is_add_possible(ct_left_i, ct_right_i) {
                return false;
            }
        }
        true
    }

    /// Computes homomorphically an addition between two ciphertexts encrypting integer values.
    ///
    /// If the operation can be performed, the result is returned in a new ciphertext.
    /// Otherwise [CheckError::CarryFull] is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 41;
    /// let msg2 = 101;
    ///
    /// let ct1 = cks.encrypt(msg1);
    /// let ct2 = cks.encrypt(msg2);
    ///
    /// // Compute homomorphically an addition:
    /// let ct_res = sks.checked_add(&ct1, &ct2);
    ///
    /// match ct_res {
    ///     Err(x) => panic!("{:?}", x),
    ///     Ok(y) => {
    ///         let clear = cks.decrypt(&y);
    ///         assert_eq!(msg1 + msg2, clear);
    ///     }
    /// }
    /// ```
    pub fn checked_add(
        &self,
        ct_left: &RadixCiphertext,
        ct_right: &RadixCiphertext,
    ) -> Result<RadixCiphertext, CheckError> {
        if self.is_add_possible(ct_left, ct_right) {
            let mut result = ct_left.clone();
            self.unchecked_add_assign(&mut result, ct_right);

            Ok(result)
        } else {
            Err(CarryFull)
        }
    }

    /// Computes homomorphically an addition between two ciphertexts encrypting integer values.
    ///
    /// If the operation can be performed, the result is stored in the `ct_left` ciphertext.
    /// Otherwise [CheckError::CarryFull] is returned, and `ct_left` is not modified.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 41;
    /// let msg2 = 101;
    ///
    /// let mut ct1 = cks.encrypt(msg1);
    /// let ct2 = cks.encrypt(msg2);
    ///
    /// // Compute homomorphically an addition:
    /// let res = sks.checked_add_assign(&mut ct1, &ct2);
    ///
    /// assert!(res.is_ok());
    ///
    /// let clear = cks.decrypt(&ct1);
    /// assert_eq!(msg1 + msg2, clear);
    /// ```
    pub fn checked_add_assign(
        &self,
        ct_left: &mut RadixCiphertext,
        ct_right: &RadixCiphertext,
    ) -> Result<(), CheckError> {
        if self.is_add_possible(ct_left, ct_right) {
            self.unchecked_add_assign(ct_left, ct_right);
            Ok(())
        } else {
            Err(CarryFull)
        }
    }

    /// Computes homomorphically an addition between two ciphertexts encrypting integer values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg1 = 14;
    /// let msg2 = 97;
    ///
    /// let mut ct1 = cks.encrypt(msg1);
    /// let mut ct2 = cks.encrypt(msg2);
    ///
    /// // Compute homomorphically an addition:
    /// let ct_res = sks.smart_add(&mut ct1, &mut ct2);
    ///
    /// // Decrypt:
    /// let dec_result = cks.decrypt(&ct_res);
    /// assert_eq!(dec_result, msg1 + msg2);
    /// ```
    pub fn smart_add(
        &self,
        ct_left: &mut RadixCiphertext,
        ct_right: &mut RadixCiphertext,
    ) -> RadixCiphertext {
        if !self.is_add_possible(ct_left, ct_right) {
            self.full_propagate(ct_left);
            self.full_propagate(ct_right);
        }
        self.unchecked_add(ct_left, ct_right)
    }

    pub fn smart_add_assign(&self, ct_left: &mut RadixCiphertext, ct_right: &mut RadixCiphertext) {
        //If the ciphertext cannot be added together without exceeding the capacity of a ciphertext
        if !self.is_add_possible(ct_left, ct_right) {
            self.full_propagate(ct_left);
            self.full_propagate(ct_right);
        }
        self.unchecked_add_assign(ct_left, ct_right);
    }
}
