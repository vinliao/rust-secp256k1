// Bitcoin secp256k1 bindings
// Written in 2014 by
//   Dawid Ciężarkiewicz
//   Andrew Poelstra
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Secp256k1
//! Rust bindings for Pieter Wuille's secp256k1 library, which is used for
//! fast and accurate manipulation of ECDSA signatures on the secp256k1
//! curve. Such signatures are used extensively by the Bitcoin network
//! and its derivatives.
//!
//! To minimize dependencies, some functions are feature-gated. To generate
//! random keys or to re-randomize a context object, compile with the "rand"
//! feature. To de/serialize objects with serde, compile with "serde".
//! **Important**: `serde` encoding is **not** the same as consensus encoding!
//!
//! Where possible, the bindings use the Rust type system to ensure that
//! API usage errors are impossible. For example, the library uses context
//! objects that contain precomputation tables which are created on object
//! construction. Since this is a slow operation (10+ milliseconds, vs ~50
//! microseconds for typical crypto operations, on a 2.70 Ghz i7-6820HQ)
//! the tables are optional, giving a performance boost for users who only
//! care about signing, only care about verification, or only care about
//! parsing. In the upstream library, if you attempt to sign a message using
//! a context that does not support this, it will trigger an assertion
//! failure and terminate the program. In `rust-secp256k1`, this is caught
//! at compile-time; in fact, it is impossible to compile code that will
//! trigger any assertion failures in the upstream library.
//!
//! ```rust
//! # #[cfg(all(feature="rand", feature="bitcoin_hashes"))] {
//! use secp256k1::rand::rngs::OsRng;
//! use secp256k1::{Secp256k1, Message};
//! use secp256k1::hashes::sha256;
//!
//! let secp = Secp256k1::new();
//! let mut rng = OsRng::new().expect("OsRng");
//! let (secret_key, public_key) = secp.generate_keypair(&mut rng);
//! let message = Message::from_hashed_data::<sha256::Hash>("Hello World!".as_bytes());
//!
//! let sig = secp.sign_ecdsa(&message, &secret_key);
//! assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());
//! # }
//! ```
//!
//! The above code requires `rust-secp256k1` to be compiled with the `rand-std` and `bitcoin_hashes`
//! feature enabled, to get access to [`generate_keypair`](struct.Secp256k1.html#method.generate_keypair)
//! Alternately, keys and messages can be parsed from slices, like
//!
//! ```rust
//! use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};
//!
//! let secp = Secp256k1::new();
//! let secret_key = SecretKey::from_slice(&[0xcd; 32]).expect("32 bytes, within curve order");
//! let public_key = PublicKey::from_secret_key(&secp, &secret_key);
//! // This is unsafe unless the supplied byte slice is the output of a cryptographic hash function.
//! // See the above example for how to use this library together with `bitcoin_hashes`.
//! let message = Message::from_slice(&[0xab; 32]).expect("32 bytes");
//!
//! let sig = secp.sign_ecdsa(&message, &secret_key);
//! assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());
//! ```
//!
//! Users who only want to verify signatures can use a cheaper context, like so:
//!
//! ```rust
//! use secp256k1::{Secp256k1, Message, ecdsa, PublicKey};
//!
//! let secp = Secp256k1::verification_only();
//!
//! let public_key = PublicKey::from_slice(&[
//!     0x02,
//!     0xc6, 0x6e, 0x7d, 0x89, 0x66, 0xb5, 0xc5, 0x55,
//!     0xaf, 0x58, 0x05, 0x98, 0x9d, 0xa9, 0xfb, 0xf8,
//!     0xdb, 0x95, 0xe1, 0x56, 0x31, 0xce, 0x35, 0x8c,
//!     0x3a, 0x17, 0x10, 0xc9, 0x62, 0x67, 0x90, 0x63,
//! ]).expect("public keys must be 33 or 65 bytes, serialized according to SEC 2");
//!
//! let message = Message::from_slice(&[
//!     0xaa, 0xdf, 0x7d, 0xe7, 0x82, 0x03, 0x4f, 0xbe,
//!     0x3d, 0x3d, 0xb2, 0xcb, 0x13, 0xc0, 0xcd, 0x91,
//!     0xbf, 0x41, 0xcb, 0x08, 0xfa, 0xc7, 0xbd, 0x61,
//!     0xd5, 0x44, 0x53, 0xcf, 0x6e, 0x82, 0xb4, 0x50,
//! ]).expect("messages must be 32 bytes and are expected to be hashes");
//!
//! let sig = ecdsa::Signature::from_compact(&[
//!     0xdc, 0x4d, 0xc2, 0x64, 0xa9, 0xfe, 0xf1, 0x7a,
//!     0x3f, 0x25, 0x34, 0x49, 0xcf, 0x8c, 0x39, 0x7a,
//!     0xb6, 0xf1, 0x6f, 0xb3, 0xd6, 0x3d, 0x86, 0x94,
//!     0x0b, 0x55, 0x86, 0x82, 0x3d, 0xfd, 0x02, 0xae,
//!     0x3b, 0x46, 0x1b, 0xb4, 0x33, 0x6b, 0x5e, 0xcb,
//!     0xae, 0xfd, 0x66, 0x27, 0xaa, 0x92, 0x2e, 0xfc,
//!     0x04, 0x8f, 0xec, 0x0c, 0x88, 0x1c, 0x10, 0xc4,
//!     0xc9, 0x42, 0x8f, 0xca, 0x69, 0xc1, 0x32, 0xa2,
//! ]).expect("compact signatures are 64 bytes; DER signatures are 68-72 bytes");
//!
//! # #[cfg(not(fuzzing))]
//! assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());
//! ```
//!
//! Observe that the same code using, say [`signing_only`](struct.Secp256k1.html#method.signing_only)
//! to generate a context would simply not compile.
//!
//! ## Crate features/optional dependencies
//!
//! This crate provides the following opt-in Cargo features:
//!
//! * `std` - use standard Rust library, enabled by default.
//! * `alloc` - use the `alloc` standard Rust library to provide heap allocations.
//! * `rand` - use `rand` library to provide random generator (e.g. to generate keys).
//! * `rand-std` - use `rand` library with its `std` feature enabled. (Implies `rand`.)
//! * `recovery` - enable functions that can compute the public key from signature.
//! * `lowmemory` - optimize the library for low-memory environments.
//! * `global-context` - enable use of global secp256k1 context. (Implies `std`, `rand-std` and
//!                      `global-context-less-secure`.)
//! * `global-context-less-secure` - enables global context without extra sidechannel protection.
//! * `serde` - implements serialization and deserialization for types in this crate using `serde`.
//!           **Important**: `serde` encoding is **not** the same as consensus encoding!
//! * `bitcoin_hashes` - enables interaction with the `bitcoin-hashes` crate (e.g. conversions).

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]


#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(all(test, feature = "unstable"), feature(test))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
pub extern crate secp256k1_sys;
pub use secp256k1_sys as ffi;

#[cfg(feature = "bitcoin_hashes")]
#[cfg_attr(docsrs, doc(cfg(feature = "bitcoin_hashes")))]
pub extern crate bitcoin_hashes as hashes;
#[cfg(all(test, feature = "unstable"))]
extern crate test;
#[cfg(any(test, feature = "rand"))]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub extern crate rand;
#[cfg(any(test))]
extern crate rand_core;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub extern crate serde;
#[cfg(all(test, feature = "serde"))]
extern crate serde_test;
#[cfg(any(test, feature = "rand"))]
use rand::Rng;
#[cfg(any(test, feature = "std"))]
extern crate core;
#[cfg(all(test, target_arch = "wasm32"))]
extern crate wasm_bindgen_test;
#[cfg(feature = "alloc")]
extern crate alloc;


#[macro_use]
mod macros;
#[macro_use]
mod secret;
mod context;
mod key;

pub mod constants;
pub mod ecdh;
pub mod ecdsa;
pub mod schnorr;
#[cfg(feature = "serde")]
mod serde_util;

pub use key::*;
pub use context::*;
use core::marker::PhantomData;
use core::{mem, fmt, str};
use ffi::{CPtr, types::AlignedType};

#[cfg(feature = "global-context-less-secure")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "global-context", feature = "global-context-less-secure"))))]
pub use context::global::SECP256K1;

#[cfg(feature = "bitcoin_hashes")]
use hashes::Hash;

// Backwards compatible changes
/// Schnorr Sig related methods
#[deprecated(since = "0.21.0", note = "Use schnorr instead.")]
pub mod schnorrsig {
    #[deprecated(since = "0.21.0", note = "Use crate::XOnlyPublicKey instead.")]
    /// backwards compatible re-export of xonly key
    pub type PublicKey = super::XOnlyPublicKey;
    /// backwards compatible re-export of keypair
    #[deprecated(since = "0.21.0", note = "Use crate::KeyPair instead.")]
    pub type KeyPair = super::KeyPair;
    /// backwards compatible re-export of schnorr signatures
    #[deprecated(since = "0.21.0", note = "Use schnorr::Signature instead.")]
    pub type Signature = super::schnorr::Signature;
}

#[deprecated(since = "0.21.0", note = "Use ecdsa::Signature instead.")]
/// backwards compatible re-export of ecdsa signatures
pub type Signature = ecdsa::Signature;

/// Trait describing something that promises to be a 32-byte random number; in particular,
/// it has negligible probability of being zero or overflowing the group order. Such objects
/// may be converted to `Message`s without any error paths.
pub trait ThirtyTwoByteHash {
    /// Converts the object into a 32-byte array
    fn into_32(self) -> [u8; 32];
}

#[cfg(feature = "bitcoin_hashes")]
#[cfg_attr(docsrs, doc(cfg(feature = "bitcoin_hashes")))]
impl ThirtyTwoByteHash for hashes::sha256::Hash {
    fn into_32(self) -> [u8; 32] {
        self.into_inner()
    }
}

#[cfg(feature = "bitcoin_hashes")]
#[cfg_attr(docsrs, doc(cfg(feature = "bitcoin_hashes")))]
impl ThirtyTwoByteHash for hashes::sha256d::Hash {
    fn into_32(self) -> [u8; 32] {
        self.into_inner()
    }
}

#[cfg(feature = "bitcoin_hashes")]
#[cfg_attr(docsrs, doc(cfg(feature = "bitcoin_hashes")))]
impl<T: hashes::sha256t::Tag> ThirtyTwoByteHash for hashes::sha256t::Hash<T> {
    fn into_32(self) -> [u8; 32] {
        self.into_inner()
    }
}

/// A (hashed) message input to an ECDSA signature
pub struct Message([u8; constants::MESSAGE_SIZE]);
impl_array_newtype!(Message, u8, constants::MESSAGE_SIZE);
impl_pretty_debug!(Message);

impl Message {
    /// **If you just want to sign an arbitrary message use `Message::from_hashed_data` instead.**
    ///
    /// Converts a `MESSAGE_SIZE`-byte slice to a message object. **WARNING:** the slice has to be a
    /// cryptographically secure hash of the actual message that's going to be signed. Otherwise
    /// the result of signing isn't a
    /// [secure signature](https://twitter.com/pwuille/status/1063582706288586752).
    #[inline]
    pub fn from_slice(data: &[u8]) -> Result<Message, Error> {
        match data.len() {
            constants::MESSAGE_SIZE => {
                let mut ret = [0u8; constants::MESSAGE_SIZE];
                ret[..].copy_from_slice(data);
                Ok(Message(ret))
            }
            _ => Err(Error::InvalidMessage)
        }
    }

    /// Constructs a `Message` by hashing `data` with hash algorithm `H`. This requires the feature
    /// `bitcoin_hashes` to be enabled.
    /// ```rust
    /// extern crate bitcoin_hashes;
    /// # extern crate secp256k1;
    /// use secp256k1::Message;
    /// use bitcoin_hashes::sha256;
    /// use bitcoin_hashes::Hash;
    ///
    /// let m1 = Message::from_hashed_data::<sha256::Hash>("Hello world!".as_bytes());
    /// // is equivalent to
    /// let m2 = Message::from(sha256::Hash::hash("Hello world!".as_bytes()));
    ///
    /// assert_eq!(m1, m2);
    /// ```
    #[cfg(feature = "bitcoin_hashes")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bitcoin_hashes")))]
    pub fn from_hashed_data<H: ThirtyTwoByteHash + hashes::Hash>(data: &[u8]) -> Self {
        <H as hashes::Hash>::hash(data).into()
    }
}

impl<T: ThirtyTwoByteHash> From<T> for Message {
    /// Converts a 32-byte hash directly to a message without error paths
    fn from(t: T) -> Message {
        Message(t.into_32())
    }
}

/// An ECDSA error
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub enum Error {
    /// Signature failed verification
    IncorrectSignature,
    /// Badly sized message ("messages" are actually fixed-sized digests; see the `MESSAGE_SIZE`
    /// constant)
    InvalidMessage,
    /// Bad public key
    InvalidPublicKey,
    /// Bad signature
    InvalidSignature,
    /// Bad secret key
    InvalidSecretKey,
    /// Bad recovery id
    InvalidRecoveryId,
    /// Invalid tweak for add_*_assign or mul_*_assign
    InvalidTweak,
    /// Didn't pass enough memory to context creation with preallocated memory
    NotEnoughMemory,
    /// Bad set of public keys
    InvalidPublicKeySum,
    /// The only valid parity values are 0 or 1.
    InvalidParityValue,
}

impl Error {
    fn as_str(&self) -> &str {
        match *self {
            Error::IncorrectSignature => "secp: signature failed verification",
            Error::InvalidMessage => "secp: message was not 32 bytes (do you need to hash?)",
            Error::InvalidPublicKey => "secp: malformed public key",
            Error::InvalidSignature => "secp: malformed signature",
            Error::InvalidSecretKey => "secp: malformed or out-of-range secret key",
            Error::InvalidRecoveryId => "secp: bad recovery id",
            Error::InvalidTweak => "secp: bad tweak",
            Error::NotEnoughMemory => "secp: not enough memory allocated",
            Error::InvalidPublicKeySum => "secp: the sum of public keys was invalid or the input vector lengths was less than 1",
            Error::InvalidParityValue => "The only valid parity values are 0 or 1",
        }
    }
}

// Passthrough Debug to Display, since errors should be user-visible
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {}


/// The secp256k1 engine, used to execute all signature operations
pub struct Secp256k1<C: Context> {
    ctx: *mut ffi::Context,
    phantom: PhantomData<C>,
    size: usize,
}

// The underlying secp context does not contain any references to memory it does not own
unsafe impl<C: Context> Send for Secp256k1<C> {}
// The API does not permit any mutation of `Secp256k1` objects except through `&mut` references
unsafe impl<C: Context> Sync for Secp256k1<C> {}

impl<C: Context> PartialEq for Secp256k1<C> {
    fn eq(&self, _other: &Secp256k1<C>) -> bool { true }
}

impl<C: Context> Eq for Secp256k1<C> { }

impl<C: Context> Drop for Secp256k1<C> {
    fn drop(&mut self) {
        unsafe {
            ffi::secp256k1_context_preallocated_destroy(self.ctx);
            C::deallocate(self.ctx as _, self.size);
        }
    }
}

impl<C: Context> fmt::Debug for Secp256k1<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<secp256k1 context {:?}, {}>", self.ctx, C::DESCRIPTION)
    }
}

impl<C: Context> Secp256k1<C> {

    /// Getter for the raw pointer to the underlying secp256k1 context. This
    /// shouldn't be needed with normal usage of the library. It enables
    /// extending the Secp256k1 with more cryptographic algorithms outside of
    /// this crate.
    pub fn ctx(&self) -> &*mut ffi::Context {
        &self.ctx
    }

    /// Returns the required memory for a preallocated context buffer in a generic manner(sign/verify/all)
    pub fn preallocate_size_gen() -> usize {
        let word_size = mem::size_of::<AlignedType>();
        let bytes = unsafe { ffi::secp256k1_context_preallocated_size(C::FLAGS) };

        (bytes + word_size - 1) / word_size
    }

    /// (Re)randomizes the Secp256k1 context for cheap sidechannel resistance;
    /// see comment in libsecp256k1 commit d2275795f by Gregory Maxwell. Requires
    /// compilation with "rand" feature.
    #[cfg(any(test, feature = "rand"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn randomize<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        self.seeded_randomize(&seed);
    }

    /// (Re)randomizes the Secp256k1 context for cheap sidechannel resistance given 32 bytes of
    /// cryptographically-secure random data;
    /// see comment in libsecp256k1 commit d2275795f by Gregory Maxwell.
    pub fn seeded_randomize(&mut self, seed: &[u8; 32]) {
        unsafe {
            let err = ffi::secp256k1_context_randomize(self.ctx, seed.as_c_ptr());
            // This function cannot fail; it has an error return for future-proofing.
            // We do not expose this error since it is impossible to hit, and we have
            // precedent for not exposing impossible errors (for example in
            // `PublicKey::from_secret_key` where it is impossible to create an invalid
            // secret key through the API.)
            // However, if this DOES fail, the result is potentially weaker side-channel
            // resistance, which is deadly and undetectable, so we take out the entire
            // thread to be on the safe side.
            assert_eq!(err, 1);
        }
    }
}

impl<C: Signing> Secp256k1<C> {
    /// Generates a random keypair. Convenience function for [`SecretKey::new`] and
    /// [`PublicKey::from_secret_key`].
    #[inline]
    #[cfg(any(test, feature = "rand"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn generate_keypair<R: Rng + ?Sized>(&self, rng: &mut R)
                                    -> (key::SecretKey, key::PublicKey) {
        let sk = key::SecretKey::new(rng);
        let pk = key::PublicKey::from_secret_key(self, &sk);
        (sk, pk)
    }
}

/// Utility function used to parse hex into a target u8 buffer. Returns
/// the number of bytes converted or an error if it encounters an invalid
/// character or unexpected end of string.
fn from_hex(hex: &str, target: &mut [u8]) -> Result<usize, ()> {
    if hex.len() % 2 == 1 || hex.len() > target.len() * 2 {
        return Err(());
    }

    let mut b = 0;
    let mut idx = 0;
    for c in hex.bytes() {
        b <<= 4;
        match c {
            b'A'..=b'F' => b |= c - b'A' + 10,
            b'a'..=b'f' => b |= c - b'a' + 10,
            b'0'..=b'9' => b |= c - b'0',
            _ => return Err(()),
        }
        if (idx & 1) == 1 {
            target[idx / 2] = b;
            b = 0;
        }
        idx += 1;
    }
    Ok(idx / 2)
}

/// Utility function used to encode hex into a target u8 buffer. Returns
/// a reference to the target buffer as an str. Returns an error if the target
/// buffer isn't big enough.
#[inline]
fn to_hex<'a>(src: &[u8], target: &'a mut [u8]) -> Result<&'a str, ()> {
    let hex_len = src.len() * 2;
    if target.len() < hex_len {
        return Err(());
    }
    const HEX_TABLE: [u8; 16] = *b"0123456789abcdef";

    let mut i = 0;
    for &b in src {
        target[i] = HEX_TABLE[usize::from(b >> 4)];
        target[i+1] = HEX_TABLE[usize::from(b & 0b00001111)];
        i +=2 ;
    }
    let result = &target[..hex_len];
    debug_assert!(str::from_utf8(result).is_ok());
    return unsafe { Ok(str::from_utf8_unchecked(result)) };
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::{RngCore, thread_rng};
    use std::str::FromStr;
    use std::marker::PhantomData;
    use ffi::types::AlignedType;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    macro_rules! hex {
        ($hex:expr) => ({
            let mut result = vec![0; $hex.len() / 2];
            from_hex($hex, &mut result).expect("valid hex string");
            result
        });
    }


    #[test]
    fn test_manual_create_destroy() {
        let ctx_full = unsafe { ffi::secp256k1_context_create(AllPreallocated::FLAGS) };
        let ctx_sign = unsafe { ffi::secp256k1_context_create(SignOnlyPreallocated::FLAGS) };
        let ctx_vrfy = unsafe { ffi::secp256k1_context_create(VerifyOnlyPreallocated::FLAGS) };

        let size = 0;
        let full: Secp256k1<AllPreallocated> = Secp256k1{ctx: ctx_full, phantom: PhantomData, size};
        let sign: Secp256k1<SignOnlyPreallocated> = Secp256k1{ctx: ctx_sign, phantom: PhantomData, size};
        let vrfy: Secp256k1<VerifyOnlyPreallocated> = Secp256k1{ctx: ctx_vrfy, phantom: PhantomData, size};

        let (sk, pk) = full.generate_keypair(&mut thread_rng());
        let msg = Message::from_slice(&[2u8; 32]).unwrap();
        // Try signing
        assert_eq!(sign.sign_ecdsa(&msg, &sk), full.sign_ecdsa(&msg, &sk));
        let sig = full.sign_ecdsa(&msg, &sk);

        // Try verifying
        assert!(vrfy.verify_ecdsa(&msg, &sig, &pk).is_ok());
        assert!(full.verify_ecdsa(&msg, &sig, &pk).is_ok());

        drop(full);drop(sign);drop(vrfy);

        unsafe { ffi::secp256k1_context_destroy(ctx_vrfy) };
        unsafe { ffi::secp256k1_context_destroy(ctx_sign) };
        unsafe { ffi::secp256k1_context_destroy(ctx_full) };
    }

    #[test]
    fn test_raw_ctx() {
        use std::mem::ManuallyDrop;

        let ctx_full = Secp256k1::new();
        let ctx_sign = Secp256k1::signing_only();
        let ctx_vrfy = Secp256k1::verification_only();

        let mut full = unsafe {Secp256k1::from_raw_all(ctx_full.ctx)};
        let mut sign = unsafe {Secp256k1::from_raw_signining_only(ctx_sign.ctx)};
        let mut vrfy = unsafe {Secp256k1::from_raw_verification_only(ctx_vrfy.ctx)};

        let (sk, pk) = full.generate_keypair(&mut thread_rng());
        let msg = Message::from_slice(&[2u8; 32]).unwrap();
        // Try signing
        assert_eq!(sign.sign_ecdsa(&msg, &sk), full.sign_ecdsa(&msg, &sk));
        let sig = full.sign_ecdsa(&msg, &sk);

        // Try verifying
        assert!(vrfy.verify_ecdsa(&msg, &sig, &pk).is_ok());
        assert!(full.verify_ecdsa(&msg, &sig, &pk).is_ok());

        unsafe {
            ManuallyDrop::drop(&mut full);
            ManuallyDrop::drop(&mut sign);
            ManuallyDrop::drop(&mut vrfy);

        }
        drop(ctx_full);
        drop(ctx_sign);
        drop(ctx_vrfy);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore] // Panicking from C may trap (SIGILL) intentionally, so we test this manually.
    fn test_panic_raw_ctx_should_terminate_abnormally() {
        let ctx_vrfy = Secp256k1::verification_only();
        let raw_ctx_verify_as_full = unsafe {Secp256k1::from_raw_all(ctx_vrfy.ctx)};
        // Generating a key pair in verify context will panic (ARG_CHECK).
        raw_ctx_verify_as_full.generate_keypair(&mut thread_rng());
    }

    #[test]
    fn test_preallocation() {
        let mut buf_ful = vec![AlignedType::zeroed(); Secp256k1::preallocate_size()];
        let mut buf_sign = vec![AlignedType::zeroed(); Secp256k1::preallocate_signing_size()];
        let mut buf_vfy = vec![AlignedType::zeroed(); Secp256k1::preallocate_verification_size()];

        let full = Secp256k1::preallocated_new(&mut buf_ful).unwrap();
        let sign = Secp256k1::preallocated_signing_only(&mut buf_sign).unwrap();
        let vrfy = Secp256k1::preallocated_verification_only(&mut buf_vfy).unwrap();

//        drop(buf_vfy); // The buffer can't get dropped before the context.
//        println!("{:?}", buf_ful[5]); // Can't even read the data thanks to the borrow checker.

        let (sk, pk) = full.generate_keypair(&mut thread_rng());
        let msg = Message::from_slice(&[2u8; 32]).unwrap();
        // Try signing
        assert_eq!(sign.sign_ecdsa(&msg, &sk), full.sign_ecdsa(&msg, &sk));
        let sig = full.sign_ecdsa(&msg, &sk);

        // Try verifying
        assert!(vrfy.verify_ecdsa(&msg, &sig, &pk).is_ok());
        assert!(full.verify_ecdsa(&msg, &sig, &pk).is_ok());
    }

    #[test]
    fn capabilities() {
        let sign = Secp256k1::signing_only();
        let vrfy = Secp256k1::verification_only();
        let full = Secp256k1::new();

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();

        // Try key generation
        let (sk, pk) = full.generate_keypair(&mut thread_rng());

        // Try signing
        assert_eq!(sign.sign_ecdsa(&msg, &sk), full.sign_ecdsa(&msg, &sk));
        let sig = full.sign_ecdsa(&msg, &sk);

        // Try verifying
        assert!(vrfy.verify_ecdsa(&msg, &sig, &pk).is_ok());
        assert!(full.verify_ecdsa(&msg, &sig, &pk).is_ok());

        // Check that we can produce keys from slices with no precomputation
        let (pk_slice, sk_slice) = (&pk.serialize(), &sk[..]);
        let new_pk = PublicKey::from_slice(pk_slice).unwrap();
        let new_sk = SecretKey::from_slice(sk_slice).unwrap();
        assert_eq!(sk, new_sk);
        assert_eq!(pk, new_pk);
    }

    #[test]
    fn signature_serialize_roundtrip() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let mut msg = [0u8; 32];
        for _ in 0..100 {
            thread_rng().fill_bytes(&mut msg);
            let msg = Message::from_slice(&msg).unwrap();

            let (sk, _) = s.generate_keypair(&mut thread_rng());
            let sig1 = s.sign_ecdsa(&msg, &sk);
            let der = sig1.serialize_der();
            let sig2 = ecdsa::Signature::from_der(&der[..]).unwrap();
            assert_eq!(sig1, sig2);

            let compact = sig1.serialize_compact();
            let sig2 = ecdsa::Signature::from_compact(&compact[..]).unwrap();
            assert_eq!(sig1, sig2);

            assert!(ecdsa::Signature::from_compact(&der[..]).is_err());
            assert!(ecdsa::Signature::from_compact(&compact[0..4]).is_err());
            assert!(ecdsa::Signature::from_der(&compact[..]).is_err());
            assert!(ecdsa::Signature::from_der(&der[0..4]).is_err());
         }
    }

    #[test]
    fn signature_display() {
        let hex_str = "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45";
        let byte_str = hex!(hex_str);

        assert_eq!(
            ecdsa::Signature::from_der(&byte_str).expect("byte str decode"),
            ecdsa::Signature::from_str(&hex_str).expect("byte str decode")
        );

        let sig = ecdsa::Signature::from_str(&hex_str).expect("byte str decode");
        assert_eq!(&sig.to_string(), hex_str);
        assert_eq!(&format!("{:?}", sig), hex_str);

        assert!(ecdsa::Signature::from_str(
            "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab4"
        ).is_err());
        assert!(ecdsa::Signature::from_str(
            "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab"
        ).is_err());
        assert!(ecdsa::Signature::from_str(
            "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eabxx"
        ).is_err());
        assert!(ecdsa::Signature::from_str(
            "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45"
        ).is_err());

        // 71 byte signature
        let hex_str = "30450221009d0bad576719d32ae76bedb34c774866673cbde3f4e12951555c9408e6ce774b02202876e7102f204f6bfee26c967c3926ce702cf97d4b010062e193f763190f6776";
        let sig = ecdsa::Signature::from_str(&hex_str).expect("byte str decode");
        assert_eq!(&format!("{}", sig), hex_str);
    }

    #[test]
    fn signature_lax_der() {
        macro_rules! check_lax_sig(
            ($hex:expr) => ({
                let sig = hex!($hex);
                assert!(ecdsa::Signature::from_der_lax(&sig[..]).is_ok());
            })
        );

        check_lax_sig!("304402204c2dd8a9b6f8d425fcd8ee9a20ac73b619906a6367eac6cb93e70375225ec0160220356878eff111ff3663d7e6bf08947f94443845e0dcc54961664d922f7660b80c");
        check_lax_sig!("304402202ea9d51c7173b1d96d331bd41b3d1b4e78e66148e64ed5992abd6ca66290321c0220628c47517e049b3e41509e9d71e480a0cdc766f8cdec265ef0017711c1b5336f");
        check_lax_sig!("3045022100bf8e050c85ffa1c313108ad8c482c4849027937916374617af3f2e9a881861c9022023f65814222cab09d5ec41032ce9c72ca96a5676020736614de7b78a4e55325a");
        check_lax_sig!("3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45");
        check_lax_sig!("3046022100eaa5f90483eb20224616775891397d47efa64c68b969db1dacb1c30acdfc50aa022100cf9903bbefb1c8000cf482b0aeeb5af19287af20bd794de11d82716f9bae3db1");
        check_lax_sig!("3045022047d512bc85842ac463ca3b669b62666ab8672ee60725b6c06759e476cebdc6c102210083805e93bd941770109bcc797784a71db9e48913f702c56e60b1c3e2ff379a60");
        check_lax_sig!("3044022023ee4e95151b2fbbb08a72f35babe02830d14d54bd7ed1320e4751751d1baa4802206235245254f58fd1be6ff19ca291817da76da65c2f6d81d654b5185dd86b8acf");
    }

    #[test]
    fn sign_and_verify_ecdsa() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let mut msg = [0u8; 32];
        for _ in 0..100 {
            thread_rng().fill_bytes(&mut msg);
            let msg = Message::from_slice(&msg).unwrap();

            let (sk, pk) = s.generate_keypair(&mut thread_rng());
            let sig = s.sign_ecdsa(&msg, &sk);
            assert_eq!(s.verify_ecdsa(&msg, &sig, &pk), Ok(()));
            let low_r_sig = s.sign_ecdsa_low_r(&msg, &sk);
            assert_eq!(s.verify_ecdsa(&msg, &low_r_sig, &pk), Ok(()));
            let grind_r_sig = s.sign_ecdsa_grind_r(&msg, &sk, 1);
            assert_eq!(s.verify_ecdsa(&msg, &grind_r_sig, &pk), Ok(()));
            let compact = sig.serialize_compact();
            if compact[0] < 0x80 {
                assert_eq!(sig, low_r_sig);
            } else {
                #[cfg(not(fuzzing))]  // mocked sig generation doesn't produce low-R sigs
                assert_ne!(sig, low_r_sig);
            }
            #[cfg(not(fuzzing))]  // mocked sig generation doesn't produce low-R sigs
            assert!(ecdsa::compact_sig_has_zero_first_bit(&low_r_sig.0));
            #[cfg(not(fuzzing))]  // mocked sig generation doesn't produce low-R sigs
            assert!(ecdsa::der_length_check(&grind_r_sig.0, 70));
         }
    }

    #[test]
    fn sign_and_verify_extreme() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        // Wild keys: 1, CURVE_ORDER - 1
        // Wild msgs: 1, CURVE_ORDER - 1
        let mut wild_keys = [[0u8; 32]; 2];
        let mut wild_msgs = [[0u8; 32]; 2];

        wild_keys[0][0] = 1;
        wild_msgs[0][0] = 1;

        use constants;
        wild_keys[1][..].copy_from_slice(&constants::CURVE_ORDER[..]);
        wild_msgs[1][..].copy_from_slice(&constants::CURVE_ORDER[..]);

        wild_keys[1][0] -= 1;
        wild_msgs[1][0] -= 1;

        for key in wild_keys.iter().map(|k| SecretKey::from_slice(&k[..]).unwrap()) {
            for msg in wild_msgs.iter().map(|m| Message::from_slice(&m[..]).unwrap()) {
                let sig = s.sign_ecdsa(&msg, &key);
                let low_r_sig = s.sign_ecdsa_low_r(&msg, &key);
                let grind_r_sig = s.sign_ecdsa_grind_r(&msg, &key, 1);
                let pk = PublicKey::from_secret_key(&s, &key);
                assert_eq!(s.verify_ecdsa(&msg, &sig, &pk), Ok(()));
                assert_eq!(s.verify_ecdsa(&msg, &low_r_sig, &pk), Ok(()));
                assert_eq!(s.verify_ecdsa(&msg, &grind_r_sig, &pk), Ok(()));
            }
        }
    }

    #[test]
    fn sign_and_verify_fail() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();

        let (sk, pk) = s.generate_keypair(&mut thread_rng());

        let sig = s.sign_ecdsa(&msg, &sk);

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();
        assert_eq!(s.verify_ecdsa(&msg, &sig, &pk), Err(Error::IncorrectSignature));
    }

    #[test]
    fn test_bad_slice() {
        assert_eq!(ecdsa::Signature::from_der(&[0; constants::MAX_SIGNATURE_SIZE + 1]),
                   Err(Error::InvalidSignature));
        assert_eq!(ecdsa::Signature::from_der(&[0; constants::MAX_SIGNATURE_SIZE]),
                   Err(Error::InvalidSignature));

        assert_eq!(Message::from_slice(&[0; constants::MESSAGE_SIZE - 1]),
                   Err(Error::InvalidMessage));
        assert_eq!(Message::from_slice(&[0; constants::MESSAGE_SIZE + 1]),
                   Err(Error::InvalidMessage));
        assert!(Message::from_slice(&[0; constants::MESSAGE_SIZE]).is_ok());
        assert!(Message::from_slice(&[1; constants::MESSAGE_SIZE]).is_ok());
    }

    #[test]
    fn test_hex() {
        let mut rng = thread_rng();
        const AMOUNT: usize = 1024;
        for i in 0..AMOUNT {
            // 255 isn't a valid utf8 character.
            let mut hex_buf = [255u8; AMOUNT*2];
            let mut src_buf = [0u8; AMOUNT];
            let mut result_buf = [0u8; AMOUNT];
            let src = &mut src_buf[0..i];
            rng.fill_bytes(src);

            let hex = to_hex(src, &mut hex_buf).unwrap();
            assert_eq!(from_hex(hex, &mut result_buf).unwrap(), i);
            assert_eq!(src, &result_buf[..i]);
        }


        assert!(to_hex(&[1;2], &mut [0u8; 3]).is_err());
        assert!(to_hex(&[1;2], &mut [0u8; 4]).is_ok());
        assert!(from_hex("deadbeaf", &mut [0u8; 3]).is_err());
        assert!(from_hex("deadbeaf", &mut [0u8; 4]).is_ok());
        assert!(from_hex("a", &mut [0u8; 4]).is_err());
        assert!(from_hex("ag", &mut [0u8; 4]).is_err());
    }

    #[test]
    #[cfg(not(fuzzing))]  // fixed sig vectors can't work with fuzz-sigs
    fn test_low_s() {
        // nb this is a transaction on testnet
        // txid 8ccc87b72d766ab3128f03176bb1c98293f2d1f85ebfaf07b82cc81ea6891fa9
        //      input number 3
        let sig = hex!("3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45");
        let pk = hex!("031ee99d2b786ab3b0991325f2de8489246a6a3fdb700f6d0511b1d80cf5f4cd43");
        let msg = hex!("a4965ca63b7d8562736ceec36dfa5a11bf426eb65be8ea3f7a49ae363032da0d");

        let secp = Secp256k1::new();
        let mut sig = ecdsa::Signature::from_der(&sig[..]).unwrap();
        let pk = PublicKey::from_slice(&pk[..]).unwrap();
        let msg = Message::from_slice(&msg[..]).unwrap();

        // without normalization we expect this will fail
        assert_eq!(secp.verify_ecdsa(&msg, &sig, &pk), Err(Error::IncorrectSignature));
        // after normalization it should pass
        sig.normalize_s();
        assert_eq!(secp.verify_ecdsa(&msg, &sig, &pk), Ok(()));
    }

    #[test]
    #[cfg(not(fuzzing))]  // fuzz-sigs have fixed size/format
    fn test_low_r() {
        let secp = Secp256k1::new();
        let msg = hex!("887d04bb1cf1b1554f1b268dfe62d13064ca67ae45348d50d1392ce2d13418ac");
        let msg = Message::from_slice(&msg).unwrap();
        let sk = SecretKey::from_str("57f0148f94d13095cfda539d0da0d1541304b678d8b36e243980aab4e1b7cead").unwrap();
        let expected_sig = hex!("047dd4d049db02b430d24c41c7925b2725bcd5a85393513bdec04b4dc363632b1054d0180094122b380f4cfa391e6296244da773173e78fc745c1b9c79f7b713");
        let expected_sig = ecdsa::Signature::from_compact(&expected_sig).unwrap();

        let sig = secp.sign_ecdsa_low_r(&msg, &sk);

        assert_eq!(expected_sig, sig);
    }

    #[test]
    #[cfg(not(fuzzing))]  // fuzz-sigs have fixed size/format
    fn test_grind_r() {
        let secp = Secp256k1::new();
        let msg = hex!("ef2d5b9a7c61865a95941d0f04285420560df7e9d76890ac1b8867b12ce43167");
        let msg = Message::from_slice(&msg).unwrap();
        let sk = SecretKey::from_str("848355d75fe1c354cf05539bb29b2015f1863065bcb6766b44d399ab95c3fa0b").unwrap();
        let expected_sig = ecdsa::Signature::from_str("304302202ffc447100d518c8ba643d11f3e6a83a8640488e7d2537b1954b942408be6ea3021f26e1248dd1e52160c3a38af9769d91a1a806cab5f9d508c103464d3c02d6e1").unwrap();

        let sig = secp.sign_ecdsa_grind_r(&msg, &sk, 2);

        assert_eq!(expected_sig, sig);
    }

    #[cfg(feature = "serde")]
    #[cfg(not(fuzzing))]  // fixed sig vectors can't work with fuzz-sigs
    #[test]
    fn test_serde() {
        use serde_test::{Configure, Token, assert_tokens};

        let s = Secp256k1::new();

        let msg = Message::from_slice(&[1; 32]).unwrap();
        let sk = SecretKey::from_slice(&[2; 32]).unwrap();
        let sig = s.sign_ecdsa(&msg, &sk);
        static SIG_BYTES: [u8; 71] = [
            48, 69, 2, 33, 0, 157, 11, 173, 87, 103, 25, 211, 42, 231, 107, 237,
            179, 76, 119, 72, 102, 103, 60, 189, 227, 244, 225, 41, 81, 85, 92, 148,
            8, 230, 206, 119, 75, 2, 32, 40, 118, 231, 16, 47, 32, 79, 107, 254,
            226, 108, 150, 124, 57, 38, 206, 112, 44, 249, 125, 75, 1, 0, 98, 225,
            147, 247, 99, 25, 15, 103, 118
        ];
        static SIG_STR: &'static str = "\
            30450221009d0bad576719d32ae76bedb34c774866673cbde3f4e12951555c9408e6ce77\
            4b02202876e7102f204f6bfee26c967c3926ce702cf97d4b010062e193f763190f6776\
        ";

        assert_tokens(&sig.compact(), &[Token::BorrowedBytes(&SIG_BYTES[..])]);
        assert_tokens(&sig.compact(), &[Token::Bytes(&SIG_BYTES)]);
        assert_tokens(&sig.compact(), &[Token::ByteBuf(&SIG_BYTES)]);

        assert_tokens(&sig.readable(), &[Token::BorrowedStr(SIG_STR)]);
        assert_tokens(&sig.readable(), &[Token::Str(SIG_STR)]);
        assert_tokens(&sig.readable(), &[Token::String(SIG_STR)]);

    }

    #[cfg(feature = "global-context-less-secure")]
    #[test]
    fn test_global_context() {
        use super::SECP256K1;

        let sk_data = hex!("e6dd32f8761625f105c39a39f19370b3521d845a12456d60ce44debd0a362641");
        let sk = SecretKey::from_slice(&sk_data).unwrap();
        let msg_data = hex!("a4965ca63b7d8562736ceec36dfa5a11bf426eb65be8ea3f7a49ae363032da0d");
        let msg = Message::from_slice(&msg_data).unwrap();

        // Check usage as explicit parameter
        let pk = PublicKey::from_secret_key(&SECP256K1, &sk);

        // Check usage as self
        let sig = SECP256K1.sign_ecdsa(&msg, &sk);
        assert!(SECP256K1.verify_ecdsa(&msg, &sig, &pk).is_ok());
    }

    #[cfg(feature = "bitcoin_hashes")]
    #[test]
    fn test_from_hash() {
        use hashes;
        use hashes::Hash;

        let test_bytes = "Hello world!".as_bytes();

        let hash = hashes::sha256::Hash::hash(test_bytes);
        let msg = Message::from(hash);
        assert_eq!(msg.0, hash.into_inner());
        assert_eq!(
            msg,
            Message::from_hashed_data::<hashes::sha256::Hash>(test_bytes)
        );

        let hash = hashes::sha256d::Hash::hash(test_bytes);
        let msg = Message::from(hash);
        assert_eq!(msg.0, hash.into_inner());
        assert_eq!(
            msg,
            Message::from_hashed_data::<hashes::sha256d::Hash>(test_bytes)
        );
    }
}

#[cfg(all(test, feature = "unstable"))]
mod benches {
    use rand::{thread_rng, RngCore};
    use test::{Bencher, black_box};

    use super::{Secp256k1, Message};

    #[bench]
    pub fn generate(bh: &mut Bencher) {
        struct CounterRng(u64);
        impl RngCore for CounterRng {
            fn next_u32(&mut self) -> u32 {
                self.next_u64() as u32
            }

            fn next_u64(&mut self) -> u64 {
                self.0 += 1;
                self.0
            }

            fn fill_bytes(&mut self, dest: &mut [u8]) {
                for chunk in dest.chunks_mut(64/8) {
                    let rand: [u8; 64/8] = unsafe {std::mem::transmute(self.next_u64())};
                    chunk.copy_from_slice(&rand[..chunk.len()]);
                }
            }

            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
                Ok(self.fill_bytes(dest))
            }
        }


        let s = Secp256k1::new();
        let mut r = CounterRng(0);
        bh.iter( || {
            let (sk, pk) = s.generate_keypair(&mut r);
            black_box(sk);
            black_box(pk);
        });
    }

    #[bench]
    pub fn bench_sign_ecdsa(bh: &mut Bencher) {
        let s = Secp256k1::new();
        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();
        let (sk, _) = s.generate_keypair(&mut thread_rng());

        bh.iter(|| {
            let sig = s.sign_ecdsa(&msg, &sk);
            black_box(sig);
        });
    }

    #[bench]
    pub fn bench_verify_ecdsa(bh: &mut Bencher) {
        let s = Secp256k1::new();
        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();
        let (sk, pk) = s.generate_keypair(&mut thread_rng());
        let sig = s.sign_ecdsa(&msg, &sk);

        bh.iter(|| {
            let res = s.verify_ecdsa(&msg, &sig, &pk).unwrap();
            black_box(res);
        });
    }
}
