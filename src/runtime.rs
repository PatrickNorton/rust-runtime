use crate::custom_types::lambda::Lambda;
use crate::executor;
use crate::file_info::FileInfo;
use crate::function::NativeFunction;
use crate::method::NativeMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::vec::Vec;

#[derive(Debug)]
pub struct Runtime {
    variables: Vec<Variable>,
    frames: Vec<StackFrame>,
    file_stack: Vec<usize>,
    exception_frames: HashMap<Variable, Vec<(u32, usize)>>,
    exception_stack: Vec<Variable>,
    completed_statics: HashSet<(usize, u16, u32)>,
    static_vars: Vec<Variable>,
    type_vars: HashMap<Type, HashMap<Name, Variable>>,
    ret_count: usize,

    files: Vec<FileInfo>,
}

#[derive(Debug)]
enum InnerException {
    Std(Variable),
    UnConstructed(Type, StringVar),
}

impl Runtime {
    pub fn new(files: Vec<FileInfo>, starting_no: usize) -> Runtime {
        Runtime {
            variables: vec![],
            frames: vec![StackFrame::new(0, 0, vec![])],
            file_stack: vec![starting_no],
            exception_frames: HashMap::new(),
            exception_stack: vec![],
            completed_statics: HashSet::new(),
            static_vars: Vec::new(),
            type_vars: HashMap::new(),
            ret_count: 0,
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
        &self.files[*self.file_stack.last().unwrap()].get_constants()[index as usize]
    }

    pub fn load_value(&self, index: u16) -> &Variable {
        &self.frames.last().unwrap()[index as usize]
    }

    pub fn store_variable(&mut self, index: u16, value: Variable) {
        self.frames.last_mut().unwrap()[index as usize] = value;
    }

    pub fn call_quick(&mut self, fn_no: u16) {
        self.frames.push(StackFrame::new(0, fn_no, Vec::new()));
    }

    pub fn tail_quick(&mut self, fn_no: u16) {
        let frame = self.frames.last_mut().unwrap();
        *frame = StackFrame::new(0, fn_no, Vec::new());
    }

    pub fn call_tos_or_goto(&mut self, argc: u16) -> FnResult {
        let args = self.load_args(argc);
        let callee = self.pop();
        callee.call_or_goto((args, self))
    }

    pub fn tail_tos_or_goto(&mut self, argc: u16) -> FnResult {
        self.frames.pop();
        self.call_tos_or_goto(argc)
    }

    pub fn call_op(&mut self, var: Variable, o: Operator, args: Vec<Variable>) -> FnResult {
        var.call_op(o, args, self)
    }

    pub fn call_attr(&mut self, var: Variable, s: StringVar, args: Vec<Variable>) -> FnResult {
        var.index(Name::Attribute(s), self)?.call((args, self))
    }

    pub fn call_native_method<T>(
        &mut self,
        func: NativeMethod<T>,
        this: &T,
        args: Vec<Variable>,
    ) -> FnResult {
        let native = self.is_native();
        if native {
            self.push_native();
            let result = func(this, args, self);
            self.pop_native();
            result
        } else {
            func(this, args, self)
        }
    }

    pub fn call_native(&mut self, func: NativeFunction, args: Vec<Variable>) -> FnResult {
        self.push_native();
        let result = func(args, self);
        self.pop_native();
        result
    }

    pub fn goto(&mut self, pos: u32) {
        self.frames.last_mut().unwrap().jump(pos)
    }

    pub fn current_fn(&self) -> &[u8] {
        self.current_file().get_functions()[self.frames.last().unwrap().get_fn_number() as usize]
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

    pub fn call_now(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info_no: usize,
    ) -> FnResult {
        self.push_native();
        self.push_stack(var_count, fn_no, args, info_no);
        let result = executor::execute(self);
        self.pop_native();
        result
    }

    pub fn call_now_with_frame(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info_no: usize,
        frame: StackFrame,
    ) -> FnResult {
        self.push_native();
        self.push_stack_with_frame(var_count, fn_no, args, info_no, frame);
        let result = executor::execute(self);
        self.pop_native();
        result
    }

    fn current_file(&self) -> &FileInfo {
        &self.files[*self.file_stack.last().unwrap()]
    }

    pub fn push_stack(&mut self, var_count: u16, fn_no: u16, args: Vec<Variable>, info_no: usize) {
        if info_no == *self.file_stack.last().unwrap() {
            self.frames.push(StackFrame::new(var_count, fn_no, args));
        } else {
            self.frames
                .push(StackFrame::new_file(var_count, fn_no, args));
            self.file_stack.push(info_no);
        }
    }

    pub fn push_stack_with_frame(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info_no: usize,
        frame: StackFrame,
    ) {
        if info_no == *self.file_stack.last().unwrap() {
            self.frames
                .push(StackFrame::from_old(var_count, fn_no, args, frame));
        } else {
            self.frames
                .push(StackFrame::from_old_new_file(var_count, fn_no, args, frame));
            self.file_stack.push(info_no);
        }
    }

    pub fn push_native(&mut self) {
        self.frames.push(StackFrame::native());
    }

    pub fn pop_native(&mut self) {
        debug_assert!(self.is_native());
        self.pop_stack();
    }

    pub fn pop_stack(&mut self) {
        for v in self.frames.last().unwrap().get_exceptions() {
            assert_eq!(
                self.exception_frames[v].last().unwrap().1,
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

    pub fn get_fn_name(&self, file_no: usize, fn_no: u32) -> StringVar {
        return self.files[file_no].get_functions()[fn_no as usize]
            .get_name()
            .clone()
            .into();
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

    pub fn do_static(&mut self) -> bool {
        let last_frame = self.frames.last().unwrap();
        assert!(!last_frame.is_native());
        let triplet = (
            *self.file_stack.last().unwrap(),
            last_frame.get_fn_number(),
            last_frame.current_pos(),
        );
        self.completed_statics.insert(triplet)
    }

    pub fn store_static(&mut self, index: usize, var: Variable) {
        self.static_vars
            .resize(max(self.static_vars.len(), index + 1), Variable::Null());
        self.static_vars[index] = var;
    }

    pub fn load_static(&mut self, index: usize) -> Variable {
        self.static_vars[index].clone()
    }

    pub fn add_exception_handler(&mut self, exception_type: Variable, jump_loc: u32) {
        match self.exception_frames.get_mut(&exception_type) {
            Option::Some(val) => val.push((jump_loc, self.frames.len())),
            Option::None => {
                self.exception_frames
                    .insert(exception_type.clone(), vec![(jump_loc, self.frames.len())]);
            }
        }
        self.frames
            .last_mut()
            .unwrap()
            .add_exception_handler(exception_type.clone());
        self.exception_stack.push(exception_type);
    }

    pub fn remove_exception_handler(&mut self, exception_type: Variable) {
        match self.exception_frames.get_mut(&exception_type) {
            Option::Some(fr) => fr.pop(),
            Option::None => panic!("{:?} not found", exception_type),
        };
        self.frames
            .last_mut()
            .unwrap()
            .remove_exception_handler(exception_type);
    }

    pub fn pop_handler(&mut self) {
        self.remove_exception_handler(self.exception_stack.last().unwrap().clone());
        self.exception_stack.pop();
    }

    pub fn load_fn(&self, fn_no: u16) -> Variable {
        Rc::new(Lambda::new(
            *self.file_stack.last().unwrap(),
            fn_no as u32,
            Rc::new(RefCell::new(self.frames.last().unwrap().clone())),
        ))
        .into()
    }

    pub(crate) fn set_ret(&mut self, ret_count: usize) {
        self.ret_count = ret_count;
    }

    pub fn return_0(&mut self) -> FnResult {
        self.ret_count = 0;
        FnResult::Ok(())
    }

    pub fn return_1(&mut self, var: Variable) -> FnResult {
        self.ret_count = 1;
        self.push(var);
        FnResult::Ok(())
    }

    pub fn return_n(&mut self, var: Vec<Variable>) -> FnResult {
        self.ret_count = var.len();
        self.variables.extend(var);
        FnResult::Ok(())
    }

    pub fn pop_return(&mut self) -> Variable {
        match self.ret_count {
            0 => panic!("Attempted to call pop_return where no values were returned"),
            1 => self.pop(),
            _ => {
                let new_len = self.variables.len() - self.ret_count + 1;
                self.ret_count = 0;
                self.variables.truncate(new_len);
                self.pop()
            }
        }
    }

    pub fn static_attr(&self, cls: &Type, name: Name) -> Variable {
        self.type_vars[cls][&name].clone()
    }

    pub fn set_static_attr(&mut self, cls: &Type, name: Name, var: Variable) {
        self.type_vars.get_mut(cls).unwrap().insert(name, var);
    }

    pub fn swap_2(&mut self) {
        let len = self.variables.len();
        self.variables.swap(len - 1, len - 2);
    }

    pub fn swap_n(&mut self, index: usize) {
        let value = self.variables.remove(self.variables.len() - 1 - index);
        self.variables.push(value);
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
