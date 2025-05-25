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

use crate::ast as pt;

use super::ast::{SourceUnit, SourceUnitPart};

/// A trait that is invoked while traversing the Solidity Semantic Tree.
/// Each method of the [Visitor] trait is a hook that can be potentially overridden.
pub trait SemanticVisitor
where
    Self: Sized,
{
    type Error: std::error::Error;

    fn visit_source_unit(&mut self, _source_unit: &mut SourceUnit) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_pragma(&mut self, _pragma: &pt::PragmaDirective) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// All [`semantic`] types, such as [Statement], should implement the [Visitable] trait
/// that accepts a trait [Visitor] implementation, which has various callback handles for Solidity
/// Parse Tree nodes.
///
/// We want to take a `&mut self` to be able to implement some advanced features in the future such
/// as modifying the Parse Tree before formatting it.
pub trait SemanticVisitable {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor;
}

impl<T> SemanticVisitable for &mut T
where
    T: SemanticVisitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor,
    {
        T::visit(self, v)
    }
}

impl<T> SemanticVisitable for Option<T>
where
    T: SemanticVisitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor,
    {
        if let Some(inner) = self.as_mut() {
            inner.visit(v)
        } else {
            Ok(())
        }
    }
}

impl<T> SemanticVisitable for Box<T>
where
    T: SemanticVisitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor,
    {
        T::visit(self, v)
    }
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

impl SemanticVisitable for SourceUnitPart {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: SemanticVisitor,
    {
        match &self.part {
            pt::SourceUnitPart::PragmaDirective(pragma) => v.visit_pragma(pragma),
            _ => Ok(()),
        }
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

impl_visitable!(SourceUnit, visit_source_unit);
