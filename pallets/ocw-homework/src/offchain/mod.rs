use codec::Encode;
use sp_std::vec::Vec;

pub(crate) fn derived_key<BN>(block_number: BN, key: &[u8]) -> Vec<u8>
where
	BN: Encode,
{
	block_number.using_encoded(|encoded_bn| {
		key.iter().chain(b"@".iter()).chain(encoded_bn).copied().collect::<Vec<u8>>()
	})
}
