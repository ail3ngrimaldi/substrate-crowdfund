#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod tipos;

use frame_support::traits::{Currency, Get};
use tipos::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type LargoMinimoNombreProyecto: Get<u32>;

		#[pallet::constant]
		type LargoMaximoNombreProyecto: Get<u32>;

		type Currency: Currency<Self::AccountId>; // Pueden no utilizarlo.
	}

	#[pallet::storage]
	pub type Proyectos<T> =
		StorageMap<_, Blake2_128Concat, BoundedString<T>, BalanceDe<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProyectoCreado { quien: T::AccountId, nombre: NombreProyecto<T> },
		ProyectoApoyado { nombre: NombreProyecto<T>, cantidad: BalanceDe<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		NombreMuyLargo,
		NombreMuyCorto,
		/// El usuario quiso apoyar un proyecto con más fondos de los que dispone.
		FondosInsuficientes,
		/// El usuario quiso apoyar un proyecto inexistente.
		ProyectoNoExiste,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Crea un proyecto.
		pub fn crear_proyecto(origen: OriginFor<T>, nombre: Vec<u8>) -> DispatchResult {
			let _quien = ensure_signed(origen)?;
		
			let nombre_acotado: Result<NombreProyecto<T>, _> = NombreProyecto::<T>::try_from(nombre.clone());
			let nombre_acotado = match nombre_acotado {
				Ok(v) => v,
				Err(_) => return Err(Error::<T>::NombreMuyLargo.into()),
			};
		
			let longitud_nombre = nombre_acotado.len() as u32;
			ensure!(longitud_nombre >= T::LargoMinimoNombreProyecto::get(), Error::<T>::NombreMuyCorto);
			ensure!(longitud_nombre <= T::LargoMaximoNombreProyecto::get(), Error::<T>::NombreMuyLargo);
		
			let balance_cero = BalanceDe::<T>::from(0u32);
			Proyectos::<T>::insert(nombre_acotado, balance_cero);
		
			Ok(())
		}
		

		pub fn apoyar_proyecto(
			origen: OriginFor<T>,
			nombre: String,
			cantidad: BalanceDe<T>,
		) -> DispatchResult {

			let remitente = ensure_signed(origen)?;

			let nombre_bounded: NombreProyecto<T> = nombre.clone().try_into().unwrap();
		
			ensure!(Proyectos::<T>::contains_key(&nombre_bounded), Error::<T>::ProyectoNoExiste);

			let saldo_actual = T::Currency::free_balance(&remitente);
			ensure!(saldo_actual >= cantidad, Error::<T>::FondosInsuficientes);

			let saldo_proyecto = Proyectos::<T>::get(&nombre_bounded);
			let nuevo_saldo_proyecto = saldo_proyecto + cantidad;
			Proyectos::<T>::insert(&nombre_bounded, nuevo_saldo_proyecto);

			T::Currency::withdraw(&remitente, cantidad, frame_support::traits::WithdrawReasons::TRANSFER, frame_support::traits::ExistenceRequirement::KeepAlive)?;
		
			Ok(())
		}
	}
}
