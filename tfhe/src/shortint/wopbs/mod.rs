//! Module with the definition of the WopbsKey (WithOut padding PBS Key).
//!
//! This module implements the generation of another server public key, which allows to compute
//! an alternative version of the programmable bootstrapping. This does not require the use of a
//! bit of padding.
//!
//! In the case where a padding bit is defined, keys are generated so that there a compatible for
//! both uses.

use crate::core_crypto::commons::parameters::*;
use crate::core_crypto::commons::traits::*;
use crate::core_crypto::entities::*;
use crate::shortint::engine::ShortintEngine;
use crate::shortint::{Ciphertext, ClientKey, Parameters, ServerKey};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

// Struct for WoPBS based on the private functional packing keyswitch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WopbsKey {
    //Key for the private functional keyswitch
    pub wopbs_server_key: ServerKey,
    pub pbs_server_key: ServerKey,
    pub cbs_pfpksk: LwePrivateFunctionalPackingKeyswitchKeyListOwned<u64>,
    pub ksk_pbs_to_wopbs: LweKeyswitchKeyOwned<u64>,
    pub param: Parameters,
}

impl WopbsKey {
    /// Generate the server key required to compute a WoPBS from the client and the server keys.
    ///
    /// #Warning
    /// Only when the classical PBS is not used in the circuit
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_1_CARRY_1;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_1_CARRY_1;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// // Generate the client key and the server key:
    /// let (mut cks, mut sks) = gen_keys(WOPBS_PARAM_MESSAGE_1_CARRY_1);
    /// let mut wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// ```
    pub fn new_wopbs_key_only_for_wopbs(cks: &ClientKey, sks: &ServerKey) -> WopbsKey {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine.new_wopbs_key_only_for_wopbs(cks, sks).unwrap()
        })
    }

    /// Generate the server key required to compute a WoPBS from the client and the server keys.
    /// # Example
    ///
    /// ```rust
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_1_CARRY_1;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_1_CARRY_1;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// // Generate the client key and the server key:
    /// let (mut cks, mut sks) = gen_keys(PARAM_MESSAGE_1_CARRY_1);
    /// let mut wopbs_key = WopbsKey::new_wopbs_key(&cks, &sks, &WOPBS_PARAM_MESSAGE_1_CARRY_1);
    /// ```
    pub fn new_wopbs_key(cks: &ClientKey, sks: &ServerKey, parameters: &Parameters) -> WopbsKey {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine.new_wopbs_key(cks, sks, parameters).unwrap()
        })
    }

    /// Generate the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// # Warning: this assumes one bit of padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// // Generate the client key and the server key:
    /// let (mut cks, mut sks) = gen_keys(PARAM_MESSAGE_2_CARRY_2);
    /// let mut wopbs_key = WopbsKey::new_wopbs_key(&cks, &sks, &WOPBS_PARAM_MESSAGE_2_CARRY_2);
    /// let message_modulus = WOPBS_PARAM_MESSAGE_2_CARRY_2.message_modulus.0 as u64;
    /// let m = 2;
    /// let mut ct = cks.encrypt(m);
    /// let lut = wopbs_key.generate_lut(&ct, |x| x * x % message_modulus);
    /// let ct_res = wopbs_key.programmable_bootstrapping(&mut sks, &mut ct, &lut);
    /// let res = cks.decrypt(&ct_res);
    /// assert_eq!(res, (m * m) % message_modulus);
    /// ```
    pub fn generate_lut<F>(&self, ct: &Ciphertext, f: F) -> Vec<u64>
    where
        F: Fn(u64) -> u64,
    {
        // The function is applied only on the message modulus bits
        let basis = ct.message_modulus.0 * ct.carry_modulus.0;
        let delta = 64 - f64::log2((basis) as f64).ceil() as u64 - 1;
        let poly_size = self.wopbs_server_key.bootstrapping_key.polynomial_size().0;
        let mut vec_lut = vec![0; poly_size];
        for (i, value) in vec_lut.iter_mut().enumerate().take(basis) {
            *value = f((i % ct.message_modulus.0) as u64) << delta;
        }
        vec_lut
    }

    /// Generate the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// # Warning: this assumes no bit of padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::wopbs::WopbsKey;
    ///
    /// // Generate the client key and the server key:
    /// let (mut cks, mut sks) = gen_keys(WOPBS_PARAM_MESSAGE_2_CARRY_2);
    /// let mut wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// let message_modulus = WOPBS_PARAM_MESSAGE_2_CARRY_2.message_modulus.0 as u64;
    /// let m = 2;
    /// let ct = cks.encrypt_without_padding(m);
    /// let lut = wopbs_key.generate_lut(&ct, |x| x * x % message_modulus);
    /// let ct_res = wopbs_key.programmable_bootstrapping_without_padding(&ct, &lut);
    /// let res = cks.decrypt_without_padding(&ct_res);
    /// assert_eq!(res, (m * m) % message_modulus);
    /// ```
    pub fn generate_lut_without_padding<F>(&self, ct: &Ciphertext, f: F) -> Vec<u64>
    where
        F: Fn(u64) -> u64,
    {
        // The function is applied only on the message modulus bits
        let basis = ct.message_modulus.0 * ct.carry_modulus.0;
        let delta = 64 - f64::log2((basis) as f64).ceil() as u64;
        let poly_size = self.wopbs_server_key.bootstrapping_key.polynomial_size().0;
        let mut vec_lut = vec![0; poly_size];
        for (i, value) in vec_lut.iter_mut().enumerate().take(basis) {
            *value = f((i % ct.message_modulus.0) as u64) << delta;
        }
        vec_lut
    }

    /// Generate the Look-Up Table homomorphically using the WoPBS approach.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs::WOPBS_PARAM_MESSAGE_3_NORM2_2;
    /// use tfhe::shortint::wopbs::WopbsKey;
    ///
    /// // Generate the client key and the server key:
    /// let (cks, sks) = gen_keys(WOPBS_PARAM_MESSAGE_3_NORM2_2);
    /// let wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// let message_modulus = 5;
    /// let m = 2;
    /// let mut ct = cks.encrypt_native_crt(m, message_modulus);
    /// let lut = wopbs_key.generate_lut_native_crt(&ct, |x| x * x % message_modulus as u64);
    /// let ct_res = wopbs_key.programmable_bootstrapping_native_crt(&mut ct, &lut);
    /// let res = cks.decrypt_message_native_crt(&ct_res, message_modulus);
    /// assert_eq!(res, (m * m) % message_modulus as u64);
    /// ```
    pub fn generate_lut_native_crt<F>(&self, ct: &Ciphertext, f: F) -> Vec<u64>
    where
        F: Fn(u64) -> u64,
    {
        // The function is applied only on the message modulus bits
        let basis = ct.message_modulus.0 * ct.carry_modulus.0;
        let nb_bit = f64::log2((basis) as f64).ceil() as u64;
        let poly_size = self.wopbs_server_key.bootstrapping_key.polynomial_size().0;
        let mut vec_lut = vec![0; poly_size];
        for i in 0..basis {
            let index_lut = (((i as u64 % basis as u64) << nb_bit) / basis as u64) as usize;
            vec_lut[index_lut] =
                (((f(i as u64) % basis as u64) as u128 * (1 << 64)) / basis as u128) as u64;
        }
        vec_lut
    }

    /// Apply the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// #Warning: this assumes one bit of padding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// // Generate the client key and the server key:
    /// let (cks, sks) = gen_keys(PARAM_MESSAGE_2_CARRY_2);
    /// let wopbs_key = WopbsKey::new_wopbs_key(&cks, &sks, &WOPBS_PARAM_MESSAGE_2_CARRY_2);
    /// let mut rng = rand::thread_rng();
    /// let message_modulus = WOPBS_PARAM_MESSAGE_2_CARRY_2.message_modulus.0;
    /// let ct = cks.encrypt(rng.gen::<u64>() % message_modulus as u64);
    /// let lut = vec![(1_u64 << 59); wopbs_key.param.polynomial_size.0];
    /// let ct_res = wopbs_key.programmable_bootstrapping(&sks, &ct, &lut);
    /// let res = cks.decrypt_message_and_carry(&ct_res);
    /// assert_eq!(res, 1);
    /// ```
    pub fn programmable_bootstrapping(
        &self,
        sks: &ServerKey,
        ct_in: &Ciphertext,
        lut: &[u64],
    ) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine
                .programmable_bootstrapping(self, sks, ct_in, lut)
                .unwrap()
        })
    }

    /// Apply the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// #Warning: this assumes one bit of padding.
    /// #Warning: to use in a WoPBS context ONLY (i.e., non compliant with classical PBS)
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs_message_carry::WOPBS_PARAM_MESSAGE_2_CARRY_2;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// // Generate the client key and the server key:
    /// let (cks, sks) = gen_keys(WOPBS_PARAM_MESSAGE_2_CARRY_2);
    /// let wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// let mut rng = rand::thread_rng();
    /// let message_modulus = WOPBS_PARAM_MESSAGE_2_CARRY_2.message_modulus.0;
    /// let ct = cks.encrypt(rng.gen::<u64>() % message_modulus as u64);
    /// let lut = vec![(1_u64 << 59); wopbs_key.param.polynomial_size.0];
    /// let ct_res = wopbs_key.wopbs(&ct, &lut);
    /// let res = cks.decrypt_message_and_carry(&ct_res);
    /// assert_eq!(res, 1);
    /// ```
    pub fn wopbs(&self, ct_in: &Ciphertext, lut: &[u64]) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| engine.wopbs(self, ct_in, lut).unwrap())
    }

    /// Apply the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::Rng;
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs::WOPBS_PARAM_MESSAGE_1_NORM2_2;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// let (cks, sks) = gen_keys(WOPBS_PARAM_MESSAGE_1_NORM2_2);
    /// let wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// let mut rng = rand::thread_rng();
    /// let ct = cks.encrypt_without_padding(rng.gen::<u64>() % 2);
    /// let lut = vec![(1_u64 << 63); wopbs_key.param.polynomial_size.0];
    /// let ct_res = wopbs_key.programmable_bootstrapping_without_padding(&ct, &lut);
    /// let res = cks.decrypt_message_and_carry_without_padding(&ct_res);
    /// assert_eq!(res, 1);
    /// ```
    pub fn programmable_bootstrapping_without_padding(
        &self,
        ct_in: &Ciphertext,
        lut: &[u64],
    ) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine
                .programmable_bootstrapping_without_padding(self, ct_in, lut)
                .unwrap()
        })
    }

    /// Apply the Look-Up Table homomorphically using the WoPBS approach.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::shortint::ciphertext::Ciphertext;
    /// use tfhe::shortint::gen_keys;
    /// use tfhe::shortint::parameters::parameters_wopbs::WOPBS_PARAM_MESSAGE_3_NORM2_2;
    /// use tfhe::shortint::wopbs::*;
    ///
    /// let (cks, sks) = gen_keys(WOPBS_PARAM_MESSAGE_3_NORM2_2);
    /// let wopbs_key = WopbsKey::new_wopbs_key_only_for_wopbs(&cks, &sks);
    /// let msg = 2;
    /// let modulus = 5;
    /// let mut ct = cks.encrypt_native_crt(msg, modulus);
    /// let lut = wopbs_key.generate_lut_native_crt(&ct, |x| x);
    /// let ct_res = wopbs_key.programmable_bootstrapping_native_crt(&mut ct, &lut);
    /// let res = cks.decrypt_message_native_crt(&ct_res, modulus);
    /// assert_eq!(res, msg);
    /// ```
    pub fn programmable_bootstrapping_native_crt(
        &self,
        ct_in: &mut Ciphertext,
        lut: &[u64],
    ) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine
                .programmable_bootstrapping_native_crt(self, ct_in, lut)
                .unwrap()
        })
    }

    /// Extract the given number of bits from a ciphertext.
    ///
    /// # Warning Experimental
    pub fn extract_bits(
        &self,
        delta_log: DeltaLog,
        ciphertext: &Ciphertext,
        num_bits_to_extract: usize,
    ) -> LweCiphertextListOwned<u64> {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine
                .extract_bits(
                    delta_log,
                    &ciphertext.ct,
                    self,
                    ExtractedBitsCount(num_bits_to_extract),
                )
                .unwrap()
        })
    }

    /// Extract the given number of bits from a ciphertext.
    ///
    /// # Warning Experimental
    pub fn extract_bits_assign<OutputCont>(
        &self,
        delta_log: DeltaLog,
        ciphertext: &Ciphertext,
        num_bits_to_extract: usize,
        output: &mut LweCiphertextList<OutputCont>,
    ) where
        OutputCont: ContainerMut<Element = u64>,
    {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine.extract_bits_assign(
                delta_log,
                &ciphertext.ct,
                self,
                ExtractedBitsCount(num_bits_to_extract),
                output,
            )
        })
    }

    /// Temporary wrapper.
    ///
    /// # Warning Experimental
    pub fn circuit_bootstrapping_vertical_packing<InputCont>(
        &self,
        vec_lut: &[Vec<u64>],
        extracted_bits_blocks: &LweCiphertextList<InputCont>,
    ) -> Vec<LweCiphertextOwned<u64>>
    where
        InputCont: Container<Element = u64>,
    {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine.circuit_bootstrapping_vertical_packing(self, vec_lut, extracted_bits_blocks)
        })
    }

    pub fn keyswitch_to_wopbs_params(&self, sks: &ServerKey, ct_in: &Ciphertext) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| {
            engine.keyswitch_to_wopbs_params(sks, self, ct_in)
        })
        .unwrap()
    }

    pub fn keyswitch_to_pbs_params(&self, ct_in: &Ciphertext) -> Ciphertext {
        ShortintEngine::with_thread_local_mut(|engine| engine.keyswitch_to_pbs_params(self, ct_in))
            .unwrap()
    }
}
