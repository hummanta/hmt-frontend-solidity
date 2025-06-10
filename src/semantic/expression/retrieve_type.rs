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

use crate::semantic::ast::{Expression, RetrieveType, Type};

impl RetrieveType for Expression {
    fn ty(&self) -> Type {
        match self {
            Expression::BoolLiteral { .. } |
            Expression::More { .. } |
            Expression::Less { .. } |
            Expression::MoreEqual { .. } |
            Expression::LessEqual { .. } |
            Expression::Equal { .. } |
            Expression::Or { .. } |
            Expression::And { .. } |
            Expression::NotEqual { .. } |
            Expression::Not { .. } |
            Expression::StringCompare { .. } => Type::Bool,
            Expression::BytesLiteral { ty, .. } |
            Expression::NumberLiteral { ty, .. } |
            Expression::RationalNumberLiteral { ty, .. } |
            Expression::StructLiteral { ty, .. } |
            Expression::ArrayLiteral { ty, .. } |
            Expression::ConstArrayLiteral { ty, .. } |
            Expression::Add { ty, .. } |
            Expression::Subtract { ty, .. } |
            Expression::Multiply { ty, .. } |
            Expression::Divide { ty, .. } |
            Expression::Modulo { ty, .. } |
            Expression::Power { ty, .. } |
            Expression::BitwiseOr { ty, .. } |
            Expression::BitwiseAnd { ty, .. } |
            Expression::BitwiseXor { ty, .. } |
            Expression::ShiftLeft { ty, .. } |
            Expression::ShiftRight { ty, .. } |
            Expression::Variable { ty, .. } |
            Expression::ConstantVariable { ty, .. } |
            Expression::StorageVariable { ty, .. } |
            Expression::Load { ty, .. } |
            Expression::GetRef { ty, .. } |
            Expression::StorageLoad { ty, .. } |
            Expression::BitwiseNot { ty, .. } |
            Expression::Negate { ty, .. } |
            Expression::ConditionalOperator { ty, .. } |
            Expression::StructMember { ty, .. } |
            Expression::AllocDynamicBytes { ty, .. } |
            Expression::PreIncrement { ty, .. } |
            Expression::PreDecrement { ty, .. } |
            Expression::PostIncrement { ty, .. } |
            Expression::PostDecrement { ty, .. } |
            Expression::Assign { ty, .. } |
            Expression::Subscript { ty, .. } |
            Expression::ZeroExt { to: ty, .. } |
            Expression::SignExt { to: ty, .. } |
            Expression::Trunc { to: ty, .. } |
            Expression::CheckingTrunc { to: ty, .. } |
            Expression::Cast { to: ty, .. } |
            Expression::BytesCast { to: ty, .. } |
            Expression::UserDefinedOperator { ty, .. } |
            Expression::InternalFunction { ty, .. } |
            Expression::ExternalFunction { ty, .. } |
            Expression::NamedMember { ty, .. } |
            Expression::StorageArrayLength { ty, .. } |
            Expression::EventSelector { ty, .. } => ty.clone(),
            Expression::ExternalFunctionCallRaw { .. } => {
                panic!("two return values");
            }
            Expression::Builtin { tys: returns, .. } |
            Expression::InternalFunctionCall { returns, .. } |
            Expression::ExternalFunctionCall { returns, .. } => {
                assert_eq!(returns.len(), 1);
                returns[0].clone()
            }
            Expression::List { list, .. } => {
                assert_eq!(list.len(), 1);

                list[0].ty()
            }
            Expression::Constructor { contract_no, .. } => Type::Contract(*contract_no),
            Expression::FormatString { .. } => Type::String,
            Expression::TypeOperator { .. } => Type::Void,
        }
    }
}
