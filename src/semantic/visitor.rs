// Copyright (c) The Hummanta Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    parser::visitor::{Visitable, Visitor},
    semantic::ast::{ContractDefinition, ContractPart, SourceUnit, SourceUnitPart},
};

/// A trait that is invoked while traversing the Solidity Semantic Tree.
/// Each method of the [Visitor] trait is a hook that can be potentially overridden.
pub trait SemanticVisitor: Visitor
where
    Self: Sized,
{
    fn visit_sema_source_unit(&mut self, source_unit: &mut SourceUnit) -> Result<(), Self::Error> {
        source_unit.parts.visit(self)?;
        source_unit.contracts.visit(self)?;

        Ok(())
    }

    fn visit_sema_source_unit_part(
        &mut self,
        part: &mut SourceUnitPart,
    ) -> Result<(), Self::Error> {
        part.annotations.visit(self)?;
        part.part.visit(self)?;

        Ok(())
    }

    fn visit_sema_contract(
        &mut self,
        contract: &mut ContractDefinition,
    ) -> Result<(), Self::Error> {
        contract.annotations.visit(self)?;
        contract.base.visit(self)?;
        contract.parts.visit(self)?;

        Ok(())
    }

    fn visit_sema_contract_part(&mut self, part: &mut ContractPart) -> Result<(), Self::Error> {
        part.annotations.visit(self)?;
        part.part.visit(self)?;

        Ok(())
    }
}

pub trait SemanticVisitable {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor;
}

impl<T> SemanticVisitable for Vec<T>
where
    T: SemanticVisitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor,
    {
        for item in self.iter_mut() {
            item.visit(v)?;
        }
        Ok(())
    }
}

macro_rules! impl_visitable {
    ($type:ty, $func:ident) => {
        impl SemanticVisitable for $type {
            fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
            where
                V: SemanticVisitor,
            {
                v.$func(self)
            }
        }
    };
}

impl_visitable!(SourceUnit, visit_sema_source_unit);
impl_visitable!(SourceUnitPart, visit_sema_source_unit_part);
impl_visitable!(ContractDefinition, visit_sema_contract);
impl_visitable!(ContractPart, visit_sema_contract_part);
