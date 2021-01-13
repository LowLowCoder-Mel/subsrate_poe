#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, Parameter,
    traits::{Randomness, Currency, LockIdentifier, LockableCurrency, ExistenceRequirement, WithdrawReason, WithdrawReasons},
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use codec::{Encode, Decode};
use sp_runtime::{ DispatchError, traits::{ AtLeast32Bit, Bounded, Member } };
use crate::linked_item::{LinkedList, LinkedItem};

mod linked_item;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode)]
pub struct Kitty {
    pub dna: [u8; 16],
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
    type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnedKittiesList<T> = LinkedList<OwnedKitties<T>, <T as frame_system::Trait>::AccountId, <T as Trait>::KittyIndex>;
pub type GroupIndex = u32;

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {

		/// 存储所有的Kitties.
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;

		/// 存储的Kitties总数.
		pub KittiesCount get(fn kitties_count): T::KittyIndex;

		/// 存储自己拥有的Kitties列表.
		pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

		/// 存储每个Kitty的拥有者.
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

		/// 获取Kitty价格. None意味着没有出售.
		pub KittyPrices get(fn kitty_price): map hasher(blake2_128_concat) T::KittyIndex => Option<BalanceOf<T>>;

		pub MemeberScore get(fn member_score):
		    double_map hasher(blake2_128_concat) GroupIndex, hasher(blake2_128_concat) T::AccountId => u32;

		pub GroupMembership get(fn group_membership): map hasher(blake2_128_concat) T::AccountId => GroupIndex;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
		InvalidKittyId,
		RequireDifferentParent,
		RequireOwner,
		NotForSale,
		PriceTooLow,
    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
		<T as Trait>::KittyIndex,
		Balance = BalanceOf<T>,
	{
        /// 创建小猫并质押了一定数量的token.
		Created(AccountId, KittyIndex),

		/// 繁衍小猫并质押一定数量的token
        Breeded(AccountId, KittyIndex),

		/// 一只小猫被转移拥有权.
		Transferred(AccountId, AccountId, KittyIndex),

		/// 一只小猫被挂单.
		Ask(AccountId, KittyIndex, Option<Balance>),

		/// 一只小猫已出售.
		Sold(AccountId, AccountId, KittyIndex, Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        /// 创建一只小猫
        #[weight = 0]
        pub fn create(origin) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            let kitty_index = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let new_kitty = Kitty{ dna: dna };

            Self::insert_kitty(&sender, kitty_index, new_kitty);

            // let lock_id: LockIdentifier = *b"kitties ";
            // // 质押一定数量token
            // T::Currency::set_lock(
            //     lock_id,
            //     &sender,
            //     amount,
            //     WithdrawReasons::except(WithdrawReason::TransactionPayment),
            // );

            Self::deposit_event(RawEvent::Created(sender, kitty_index));

            Ok(())
        }

        /// 繁殖小猫
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

            // 质押一定数量token
            // let lock_id: LockIdentifier = *b"kitties ";
			// T::Currency::set_lock(
			//     lock_id,
			//     &sender,
			//     amount,
			//     WithdrawReasons::except(WithdrawReason::TransactionPayment),
			// );

			Self::deposit_event(RawEvent::Breeded(sender, new_kitty_id));

			Ok(())
		}

		/// 将小猫转让给其他人
		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::RequireOwner);

			Self::do_transfer(&sender, &to, kitty_id);

			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));

			Ok(())
		}

        /// 为自己的小猫设置卖价
        /// None 表示下架小猫
		#[weight = 0]
 		pub fn ask(origin, kitty_id: T::KittyIndex, new_price: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::RequireOwner);

			<KittyPrices<T>>::mutate_exists(kitty_id, |price| *price = new_price);

			Self::deposit_event(RawEvent::Ask(sender, kitty_id, new_price));

			Ok(())
		}

		/// 买一只小猫
		#[weight = 0]
		pub fn buy(origin, kitty_id: T::KittyIndex, price: BalanceOf<T>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			let kitty_price = Self::kitty_price(kitty_id).ok_or(Error::<T>::NotForSale)?;

			ensure!(price >= kitty_price, Error::<T>::PriceTooLow);

			T::Currency::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

			<KittyPrices<T>>::remove(kitty_id);

			Self::do_transfer(&owner, &sender, kitty_id);

			Self::deposit_event(RawEvent::Sold(owner, sender, kitty_id, kitty_price));

			Ok(())
		}
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
        <OwnedKittiesList<T>>::append(owner, kitty_id);
        <KittyOwners<T>>::insert(kitty_id, owner);
    }

    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        // 创建一只小猫并放入数据库
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesCount::<T>::put(kitty_id + 1.into());

        Self::insert_owned_kitty(owner, kitty_id);
    }

    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_1))), Error::<T>::RequireOwner);
        ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_2))), Error::<T>::RequireOwner);
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.dna;
        let kitty2_dna = kitty2.dna;

        // 生成一个128位的随机数
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        // 通过一对小猫的dna生成新的dna
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }

        Self::insert_kitty(sender, kitty_id, Kitty{ dna: new_dna });

        Ok(kitty_id)
    }

    fn do_transfer(from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex) {
        <OwnedKittiesList<T>>::remove(&from, kitty_id);
        Self::insert_owned_kitty(&to, kitty_id);
    }
}
