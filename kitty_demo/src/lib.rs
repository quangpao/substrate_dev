#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::inherent::Vec;
use frame_support::traits::Randomness;
// use frame_support::storage::bounded_vec;
// use scale_info::prelude::*;
use frame_support::traits::Currency;
use frame_support::traits::UnixTime;
// use frame_support::traits::StorageInstance;
type BalanceOf<T> = <<T as Config>::KittyCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
// type CreateDate<T> = <<T as Config>::TimeProvider as UnixTime<<T as frame_system::Config>::BlockNumber>>::Timestamp;

// impt<T: Config> Time for Pallet<T> {
// 	type Moment = T::Moment;

// 	fn now() -> Self::Moment {
// 		Self::now()
// 	}
// }

// impl<T: Config> UnixTime for Pallet<T> {
// 	fn now() -> core::time::Duration {
// 		let now = Self::now();
// 		sp_std::if_std! {
// 			if now 	== T::Moment::zero() {
// 				log::error!(
// 					target: "runtime::timestamp",
// 					"`pallet-timestamp::UnixTime::now` is called at the genesis, invalid value returned: 0",
// 				);
// 			}
// 		}
// 		core::time::Duration::from_millis(now.saturated_into::<u64>())
// 	}
// }



#[frame_support::pallet]
pub mod pallet {

	pub use super::*;

	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitties<T: Config> {
		id: Id,
		dna: Vec<u8>,
		owner: T::AccountId,
		price: BalanceOf<T>,
		gender: Gender,
		create_date: u64,
	}
	pub type Id = u32;

	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender{
		fn default() -> Self {
			Gender::Male
		}
	}


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type KittyCurrency: Currency<Self::AccountId>;
		type TimeProvider: UnixTime;
		// type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		#[pallet::constant]
		type MaxAddend: Get<u32>;
		// type ClearFrequency: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);


	//
	//
	//
	#[pallet::storage]
	#[pallet::getter(fn kitty_number)]
	pub type KittyNumber<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_number)]
	pub type Nonce<T> = StorageValue<_, u32, ValueQuery>;



	#[pallet::storage]
	#[pallet::getter(fn kitty)]
	pub(super) type Kitty<T: Config> = StorageMap<_, Blake2_128Concat, Id, Kitties<T>, OptionQuery>;


	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub(super) type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Kitties<T>>, OptionQuery>;

	//
	

	//

	//
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]	
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyStore(Vec<u8>, u32),

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;
			
			//Check if the kitties are full or not
			let kitty_owner = <KittyOwner<T>>::get(who.clone());
			let mut kitty_owner = match kitty_owner {
				Some(k) => k,
				None => Vec::new(),
			};
			ensure!(kitty_owner.len() < <T as Config>::MaxAddend::get().try_into().unwrap(), Error::<T>::StorageOverflow);

			//check the total balance of who for debug
			log::info!("total_balance: {:?}", T::KittyCurrency::total_balance(&who));

			// let nonce = Self::get_and_increase_nonce();
			// let (randomValue, _) = T::MyRandomness::random(&nonce);
			// let dna = randomValue.as_ref().to_vec();

			let current_id = <KittyNumber<T>>::get() + 1;
			let gender = Self:: gen_gender(dna.clone()).unwrap();


			let kitty = Kitties {
				id: current_id,
				dna: dna.clone(),
				owner: who.clone(),
				price: 0u32.into(),
				gender: gender,
				create_date: T::TimeProvider::now().as_secs(),
			};

			<Kitty<T>>::insert(current_id, kitty);
			<KittyNumber<T>>::put(current_id);


			let kitty = <Kitty<T>>::get(current_id);
			kitty_owner.push(kitty.unwrap());
			<KittyOwner<T>>::insert(who.clone(), kitty_owner);
			Self::deposit_event(Event::KittyStore(dna, 0u32.into()));

			Ok(().into())


		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(origin: OriginFor<T>, kitty_id: Id, new_owner: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty = <Kitty<T>>::get(kitty_id);
			ensure!(kitty.is_some(), "This kitty does not exist");
			let mut kitty = kitty.unwrap();
			ensure!(kitty.owner == sender, "You do not own this kitty");
			let kitty_owner = <KittyOwner<T>>::get(sender.clone());
			let mut kitty_owner = match kitty_owner{
				Some(k) => k,
				None => Vec::new(),
			};

			// Hàm retain dùng để giữ lại những giá trị thoả điều kiệu -> Loại bỏ giá trị không thoả
			kitty_owner.retain(|k| k.id != kitty.id.clone());
			kitty.owner = new_owner.clone();
			<KittyOwner<T>>::insert(sender.clone(), kitty_owner);
			let kitty_owner = <KittyOwner<T>>::get(new_owner.clone());
			let mut kitty_owner = match kitty_owner{
				Some(k) => k,
				None => Vec::new(),
			};
			ensure!(kitty_owner.len() < <T as Config>::MaxAddend::get().try_into().unwrap(), Error::<T>::StorageOverflow);
			kitty_owner.push(kitty);
			<KittyOwner<T>>::insert(new_owner.clone(), kitty_owner);
			Ok(())
		}


		



	}


	
}

impl<T> Pallet<T> {
	fn gen_gender(name: Vec<u8>) -> Result<Gender, Error<T>>{
		let mut res = Gender::Male;
		if name.len() % 2 == 1 {
			res = Gender::Female;
		}
		Ok(res)
	}

	// fn get_and_increase_nonce() -> Vec<u8> {
	// 	let nonce = Nonce::<T>::get();
	// 	Nonce::<T>::put(nonce + 1);
	// 	nonce.encode()
	// }
}

