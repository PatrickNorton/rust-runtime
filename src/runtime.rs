use std::rc::Rc;
use std::vec::Vec;

use crate::executor;
use crate::file_info::FileInfo;
use crate::operator::Operator;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Runtime {
    variables: Vec<Variable>,
    frames: Vec<StackFrame>,
    file_stack: Vec<Rc<FileInfo>>,
    exception_frames: HashMap<Variable, Vec<(u32, usize)>>,
    exception_stack: Vec<Variable>,

    files: Vec<Rc<FileInfo>>,
}

#[derive(Debug)]
enum InnerException {
    Std(Variable),
    UnConstructed(Type, StringVar),
}

impl Runtime {
    pub fn new(files: Vec<Rc<FileInfo>>, starting_no: usize) -> Runtime {
        Runtime {
            variables: vec![],
            frames: vec![StackFrame::new(0, 0, vec![])],
            file_stack: vec![files[starting_no].clone()],
            exception_frames: HashMap::new(),
            exception_stack: vec![],
            files,
        }
    }

    pub fn push(&mut self, var: Variable) {
        self.variables.push(var)
    }

    pub fn pop(&mut self) -> Variable {
        self.variables.pop().unwrap()
    }

    pub fn pop_bool(&mut self) -> Result<bool, ()> {
        self.variables.pop().unwrap().to_bool(self)
    }

    pub fn top(&mut self) -> &Variable {
        self.variables.last().unwrap()
    }

    pub fn load_const(&self, index: u16) -> &Variable {
        &self.file_stack.last().unwrap().get_constants()[index as usize]
    }

    pub fn load_value(&self, index: u16) -> &Variable {
        &self.frames.last().unwrap()[index as usize]
    }

    pub fn store_variable(&mut self, index: u16, value: Variable) {
        self.frames.last_mut().unwrap()[index as usize] = value;
    }

    pub fn call_quick(&mut self, fn_no: u16) -> FnResult {
        self.push_stack_with_file(0, fn_no, vec![], self.file_stack.last().unwrap().clone())
    }

    pub fn call_tos(&mut self, argc: u16) -> FnResult {
        let args = self.load_args(argc);
        let callee = self.pop();
        callee.call((args, self))
    }

    pub fn call_op(&mut self, var: Variable, o: Operator, args: Vec<Variable>) -> FnResult {
        var.index(Name::Operator(o)).call((args, self))
    }

    pub fn goto(&mut self, pos: u32) {
        self.frames.last_mut().unwrap().jump(pos)
    }

    pub fn current_fn(&self) -> &Vec<u8> {
        self.file_stack.last().unwrap().get_functions()
            [self.frames.last().unwrap().get_fn_number() as usize]
            .get_bytes()
    }

    pub fn current_pos(&self) -> usize {
        self.frames.last().unwrap().current_pos() as usize
    }

    pub fn advance(&mut self, pos: u32) {
        self.frames.last_mut().unwrap().advance(pos);
    }

    pub fn load_args(&mut self, argc: u16) -> Vec<Variable> {
        let mut args: VecDeque<Variable> = VecDeque::with_capacity(argc as usize);
        for _ in 0..argc {
            args.push_front(self.pop());
        }
        return args.into();
    }

    pub fn push_stack(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info: usize,
    ) -> FnResult {
        self.push_stack_with_file(var_count, fn_no, args, self.files[info].clone())
    }

    pub fn push_native(&mut self) {
        self.frames.push(StackFrame::native());
    }

    fn push_stack_with_file(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info: Rc<FileInfo>,
    ) -> FnResult {
        let native = self.is_native();
        if Rc::ptr_eq(&info, self.file_stack.last().unwrap()) {
            self.frames.push(StackFrame::new(var_count, fn_no, args));
        } else {
            self.frames
                .push(StackFrame::new_file(var_count, fn_no, args));
            self.file_stack.push(info);
        }
        if native {
            executor::execute(self)?;
            assert!(self.is_native());
        }
        Result::Ok(())
    }

    pub fn pop_stack(&mut self) {
        for v in self.frames.last().unwrap().get_exceptions() {
            assert_eq!(
                self.exception_frames[v].last().unwrap().1 as usize,
                self.frames.len() - 1
            );
            self.exception_frames.get_mut(v).unwrap().pop();
            self.exception_stack.pop();
        }
        if self.frames.last().unwrap().is_new_file() {
            self.file_stack.pop();
        }
        self.frames.pop();
    }

    pub fn is_native(&self) -> bool {
        self.frames.last().unwrap().is_native()
    }

    pub fn is_bottom_stack(&self) -> bool {
        self.frames.len() == 1
    }

    pub fn throw(&mut self, exception: Variable) -> FnResult {
        let frame = self
            .exception_frames
            .get(&Variable::Type(exception.get_type()));
        match frame {
            Option::Some(vec) => match vec.last() {
                Option::Some(pair) => {
                    let pair2 = pair.clone();
                    self.unwind_to_height(pair2.0, pair2.1, InnerException::Std(exception))
                }
                Option::None => panic!("{}", exception.str(self).unwrap()),
            },
            Option::None => panic!("{}", exception.str(self).unwrap()),
        }
    }

    pub fn throw_quick(&mut self, exc_type: Type, message: StringVar) -> FnResult {
        let frame = self.exception_frames.get(&Variable::Type(exc_type.clone()));
        match frame {
            Option::Some(vec) => match vec.last() {
                Option::Some(pair) => {
                    let pair2 = pair.clone();
                    self.unwind_to_height(
                        pair2.0,
                        pair2.1,
                        InnerException::UnConstructed(exc_type, message),
                    )
                }
                Option::None => panic!("{}", message),
            },
            Option::None => panic!("{}", message),
        }
    }

    fn unwind_to_height(
        &mut self,
        location: u32,
        frame_height: usize,
        exception: InnerException,
    ) -> FnResult {
        while self.frames.len() > frame_height {
            let last_frame = self.frames.last().unwrap();
            if last_frame.is_native() {
                let true_exc = match exception {
                    InnerException::Std(e) => e,
                    InnerException::UnConstructed(t, s) => {
                        t.create_inst(vec![Variable::String(s)], self).unwrap()
                    }
                };
                self.push(true_exc);
                return FnResult::Err(());
            } else {
                self.pop_stack();
            }
        }
        self.goto(location);
        FnResult::Ok(())
    }
}
