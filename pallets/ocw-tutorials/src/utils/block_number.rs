use crate::Config;
use sp_runtime::traits::Zero;

pub(crate) fn is_odd_block_number<T: Config>(block_number: T::BlockNumber) -> bool {
	return block_number % 2u32.into() != Zero::zero()
}
