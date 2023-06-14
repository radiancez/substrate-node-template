use codec::MaxEncodedLen;
use core::fmt::Debug;
use frame_support::Parameter;
use scale_info::TypeInfo;
use sp_runtime::traits::{
	AtLeast32BitUnsigned, Bounded, MaybeDisplay, MaybeSerializeDeserialize, Member,
};
use sp_std::vec::Vec;

pub(crate) fn derived_key<BN>(block_number: BN, key: &[u8]) -> Vec<u8>
where
	BN: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ AtLeast32BitUnsigned
		+ Default
		+ Bounded
		+ Copy
		+ sp_std::hash::Hash
		+ sp_std::str::FromStr
		+ MaxEncodedLen
		+ TypeInfo,
{
	block_number.using_encoded(|encoded_bn| {
		key.iter().chain(b"@".iter()).chain(encoded_bn).copied().collect::<Vec<u8>>()
	})
}
