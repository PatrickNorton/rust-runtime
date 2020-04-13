use std::vec::Vec;

use crate::variable::Variable;

pub struct Runtime {
    variables: Vec<Variable>
}

impl Runtime {
    pub(crate) fn push(&mut self, var: Variable) {
        self.variables.push(var)
    }

    pub(crate) fn pop(&mut self) -> Variable {
        self.variables.pop().unwrap()
    }

    fn call(&mut self) {
        
    }
}
