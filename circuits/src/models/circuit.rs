// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::models::*;

use snarkvm_curves::bls12_377::Fr;

use once_cell::unsync::OnceCell;
use std::{cell::RefCell, rc::Rc};

thread_local! {
    static CB: OnceCell<RefCell<Circuit >> = OnceCell::new();
}

#[derive(Clone)]
pub struct Circuit(CircuitScope<Fr>);

impl Circuit {
    fn cs() -> CircuitScope<<Self as Environment>::Field> {
        CB.with(|cb| {
            cb.get_or_init(|| {
                let scope = CircuitScope::<<Self as Environment>::Field>::new(
                    Rc::new(RefCell::new(ConstraintSystem::new())),
                    format!("ConstraintSystem::new"),
                    None,
                );
                RefCell::new(Circuit(scope))
            })
            .borrow()
            .0
            .clone()
        })
    }

    #[cfg(test)]
    pub fn reset_circuit() {
        CB.with(|cb| {
            (*cb.get().unwrap().borrow_mut()).0 = CircuitScope::<<Self as Environment>::Field>::new(
                Rc::new(RefCell::new(ConstraintSystem::new())),
                format!("ConstraintSystem::new"),
                None,
            );
        });

        assert_eq!(0, Self::cs().num_constants());
        assert_eq!(1, Self::cs().num_public());
        assert_eq!(0, Self::cs().num_private());
        assert_eq!(0, Self::cs().num_constraints());
    }

    pub fn print_circuit() {
        println!("{:?}", Self::cs().circuit.borrow());
    }
}

impl Environment for Circuit {
    type Field = Fr;

    fn new_variable(mode: Mode, value: Self::Field) -> Variable<Self::Field> {
        match mode {
            Mode::Constant => Self::cs().new_constant(value),
            Mode::Public => Self::cs().new_public(value),
            Mode::Private => Self::cs().new_private(value),
        }
    }

    fn zero() -> LinearCombination<Self::Field> {
        LinearCombination::zero()
    }

    fn one() -> LinearCombination<Self::Field> {
        LinearCombination::one()
    }

    fn is_satisfied() -> bool {
        Self::cs().is_satisfied()
    }

    fn scope(name: &str) -> CircuitScope<Self::Field> {
        CB.with(|cb| {
            let scope = Self::cs().scope(name);
            (*cb.get().unwrap().borrow_mut()).0 = scope.clone();
            scope
        })
    }

    fn scoped<Fn>(name: &str, logic: Fn)
    where
        Fn: FnOnce(CircuitScope<Self::Field>) -> (),
    {
        CB.with(|cb| {
            // Fetch the current environment.
            let current = Self::cs().clone();

            // Set the entire environment to the new scope, and run the logic.
            let scope = current.clone().scope(name);
            (*cb.get().unwrap().borrow_mut()).0 = scope.clone();
            logic(scope);

            // Return the entire environment to the previous scope.
            (*cb.get().unwrap().borrow_mut()).0 = current;
        });
    }

    fn enforce<Fn, A, B, C>(constraint: Fn)
    where
        Fn: FnOnce() -> (A, B, C),
        A: Into<LinearCombination<Self::Field>>,
        B: Into<LinearCombination<Self::Field>>,
        C: Into<LinearCombination<Self::Field>>,
    {
        Self::cs().enforce(constraint)
    }

    fn num_constants() -> usize {
        Self::cs().num_constants()
    }

    fn num_public() -> usize {
        Self::cs().num_public()
    }

    fn num_private() -> usize {
        Self::cs().num_private()
    }

    fn num_constraints() -> usize {
        Self::cs().num_constraints()
    }

    fn halt<T>(message: &'static str) -> T {
        eprintln!("{}", message);
        panic!("{}", message)
    }
}