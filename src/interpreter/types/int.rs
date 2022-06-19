use super::{string::MshString, BinaryOperator, MshValue};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct MshInt {
    pub value: isize,
}
impl MshInt {
    pub fn new(value: isize) -> Self {
        Self { value }
    }
}
impl MshValue for MshInt {
    fn objtype(&self) -> Arc<RwLock<dyn MshValue>> {
        Arc::new(RwLock::new(MshString::from("int")))
    }

    fn to_string(&self) -> Result<MshString, Arc<RwLock<dyn MshValue>>> {
        Ok(MshString::from(self.value.to_string()))
    }

    fn dot(
        &self,
        _identifier: &str,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        todo!()
    }

    fn binop(
        &self,
        other: Arc<RwLock<dyn MshValue>>,
        operator: BinaryOperator,
    ) -> Result<Arc<RwLock<dyn MshValue>>, Arc<RwLock<dyn MshValue>>> {
        match operator {
            BinaryOperator::And => todo!(),
            BinaryOperator::Or => todo!(),
            BinaryOperator::BitAnd => todo!(),
            BinaryOperator::BitOr => todo!(),
            BinaryOperator::Xor => todo!(),
            BinaryOperator::AtOperator => todo!(),
            BinaryOperator::Pow => todo!(),
            BinaryOperator::Mul => todo!(),
            BinaryOperator::Div => todo!(),
            BinaryOperator::Mod => todo!(),
            BinaryOperator::Plus => todo!(),
            BinaryOperator::Minus => todo!(),
        }
    }
}
