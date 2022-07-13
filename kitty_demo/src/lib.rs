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
use scale_info::prelude::*;

#[frame_support::pallet]
pub mod pallet {

	pub use super::*;

	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitties<T: Config> {
		id: Id,
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
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
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {

			let who = ensure_signed(origin)?;
			let mut current_id = <KittyNumber<T>>::get() + 1;

			let gender = Self:: gen_gender(dna.clone()).unwrap();
			let kitty = Kitties {
				id: current_id,
				dna: dna.clone(),
				owner: who.clone(),
				price: price,
				gender: gender,

			};

			let mut current_id = <KittyNumber<T>>::get() + 1;

			<Kitty<T>>::insert(current_id, kitty);
			<KittyNumber<T>>::put(current_id);

			let kitty_owner = <KittyOwner<T>>::get(who.clone());
			let mut kitty_owner = match kitty_owner {
				Some(mut k) => k,
				None => Vec::new(),
			};
			let kitty = <Kitty<T>>::get(current_id);
			kitty_owner.push(kitty.unwrap());
			<KittyOwner<T>>::insert(who.clone(), kitty_owner);
			Self::deposit_event(Event::KittyStore(dna, price));

			Ok(())


		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(origin: OriginFor<T>, kitty_id: Id, new_owner: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty = <Kitty<T>>::get(kitty_id);
			ensure!(kitty.is_some(), "This kitty does not exist");
			let mut kitty = kitty.unwrap();
			ensure!(kitty.owner == sender, "You do not own this kitty");
			let mut kitty_owner = <KittyOwner<T>>::get(sender.clone());
			let mut kitty_owner = match kitty_owner{
				Some(mut k) => k,
				None => Vec::new(),
			};

			// Hàm retain dùng để giữ lại những giá trị thoả điều kiệu -> Loại bỏ giá trị không thoả
			kitty_owner.retain(|k| k.id != kitty.id.clone());
			kitty.owner = new_owner.clone();
			<KittyOwner<T>>::insert(sender.clone(), kitty_owner);
			let mut kitty_owner = <KittyOwner<T>>::get(new_owner.clone());
			let mut kitty_owner = match kitty_owner{
				Some(mut k) => k,
				None => Vec::new(),
			};
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
}