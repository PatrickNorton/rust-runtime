use crate::custom_types::coroutine::Generator;
use crate::custom_types::exceptions::invalid_state;
use crate::custom_types::lambda::Lambda;
use crate::executor;
use crate::file_info::FileInfo;
use crate::function::NativeFunction;
use crate::jump_table::JumpTable;
use crate::method::NativeMethod;
use crate::name::Name;
use crate::name_map::NameMap;
use crate::operator::Operator;
use crate::stack_frame::{frame_strings, SFInfo, StackFrame};
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cmp::{max, min, Ordering};
use std::collections::{HashMap, HashSet};
use std::mem::{replace, take};
use std::rc::Rc;
use std::vec::Vec;

#[derive(Debug)]
pub struct Runtime {
    variables: Vec<Variable>,
    frames: Vec<StackFrame>,
    exception_frames: HashMap<Variable, Vec<(u32, usize)>>,
    exception_stack: Vec<Variable>,
    completed_statics: HashSet<(usize, u16, u32)>,
    static_vars: Vec<Variable>,
    type_vars: HashMap<Type, NameMap<Variable>>,
    ret_count: usize,
    borrowed_iterators: Vec<Rc<Generator>>,
    thrown_exception: Option<InnerException>,

    files: Vec<FileInfo>,
}

#[derive(Debug)]
enum InnerException {
    Std(Variable, Vec<SFInfo>),
    UnConstructed(Type, StringVar, Vec<SFInfo>),
}

#[derive(Debug)]
struct DeconstructedExc {
    cls: Type,
    msg: StringVar,
    frames: Vec<SFInfo>,
}

impl Runtime {
    pub fn new(files: Vec<FileInfo>, starting_no: usize) -> Runtime {
        Runtime {
            variables: vec![],
            frames: vec![StackFrame::new(0, 0, starting_no, vec![], 0)],
            exception_frames: HashMap::new(),
            exception_stack: vec![],
            completed_statics: HashSet::new(),
            static_vars: Vec::new(),
            type_vars: HashMap::new(),
            ret_count: 0,
            borrowed_iterators: Vec::new(),
            thrown_exception: Option::None,
            files,
        }
    }

    pub fn push(&mut self, var: Variable) {
        self.variables.push(var)
    }

    pub fn extend(&mut self, vars: impl IntoIterator<Item = Variable>) {
        self.variables.extend(vars)
    }

    pub fn pop(&mut self) -> Variable {
        self.variables.pop().expect("pop() called on empty stack")
    }

    pub fn pop_bool(&mut self) -> Result<bool, ()> {
        self.pop().into_bool(self)
    }

    pub fn top(&mut self) -> &Variable {
        self.variables.last().expect("top() called on empty stack")
    }

    pub fn load_const(&self, index: u16) -> &Variable {
        let file_no = self.current_file_no();
        &self.files[file_no].get_constants()[index as usize]
    }

    fn last_frame(&self) -> &StackFrame {
        self.frames
            .last()
            .expect("Frame stack should never be empty")
    }

    fn last_mut_frame(&mut self) -> &mut StackFrame {
        self.frames
            .last_mut()
            .expect("Frame stack should never be empty")
    }

    pub fn load_value(&self, index: u16) -> &Variable {
        #[cfg(debug_assertions)]
        if index as usize >= self.last_frame().len() {
            panic!(
                "Index {} out of bounds for len {}\n{}",
                index,
                self.last_frame().len(),
                self.frame_strings()
            )
        }
        &self.last_frame()[index as usize]
    }

    pub fn store_variable(&mut self, index: u16, value: Variable) {
        self.last_mut_frame()[index as usize] = value;
    }

    pub fn call_quick(&mut self, fn_no: u16, argc: u16) {
        let file_no = self.current_file_no();
        let start = self.variables.len() - argc as usize;
        let vars = self.variables.drain(start..).collect();
        self.push_stack(0, fn_no, vars, file_no);
    }

    pub fn tail_quick(&mut self, fn_no: u16, argc: u16) {
        let len = self.variables.len();
        let file_no = self.current_file_no();
        let frame = self // Can't use last_mut_frame here b/c of borrow-checker
            .frames
            .last_mut()
            .expect("Frame stack should never be empty");
        if frame.get_exceptions().is_empty() {
            let height = frame.original_stack_height();
            let args = self.variables.drain(len - argc as usize..).collect();
            self.variables.truncate(height);
            *frame = StackFrame::new(0, fn_no, file_no, args, height);
        } else {
            // Non-empty exception handler may require variables existing on the stack,
            // so tail-call isn't valid
            self.call_quick(fn_no, argc)
        }
    }

    pub fn call_tos_or_goto(&mut self, argc: u16) -> FnResult {
        let args = self.load_args(argc as usize);
        let callee = self.pop();
        callee.call_or_goto((args, self))
    }

    pub fn tail_tos_or_goto(&mut self, argc: u16) -> FnResult {
        let frame = self.frames.pop().unwrap();
        if frame.get_exceptions().is_empty() {
            let height = frame.original_stack_height();
            let args = self.load_args(argc as usize);
            let callee = self.pop();
            self.variables.truncate(height);
            callee.call_or_goto((args, self))
        } else {
            // Non-empty exception handler may require variables existing on the stack,
            // so tail-call isn't valid
            self.frames.push(frame);
            self.call_tos_or_goto(argc)
        }
    }

    pub fn call_op(&mut self, var: Variable, o: Operator, args: Vec<Variable>) -> FnResult {
        var.call_op(o, args, self)
    }

    pub fn call_attr(&mut self, var: Variable, s: &str, args: Vec<Variable>) -> FnResult {
        var.index(Name::Attribute(s), self)?.call((args, self))
    }

    pub fn call_native_method<T>(
        &mut self,
        func: NativeMethod<T>,
        this: T,
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
        self.last_mut_frame().jump(pos)
    }

    pub fn current_fn(&self) -> &[u8] {
        self.current_file().get_functions()[self.last_frame().get_fn_number() as usize].get_bytes()
    }

    pub fn current_pos(&self) -> usize {
        self.last_frame().current_pos() as usize
    }

    pub fn advance(&mut self, pos: u32) {
        self.last_mut_frame().advance(pos);
    }

    pub fn load_args(&mut self, argc: usize) -> Vec<Variable> {
        self.variables
            .drain(self.variables.len() - argc..)
            .collect()
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

    fn current_file_no(&self) -> usize {
        self.last_frame().file_no()
    }

    fn current_file(&self) -> &FileInfo {
        &self.files[self.current_file_no()]
    }

    pub(crate) fn file_no(&self, file_no: usize) -> &FileInfo {
        &self.files[file_no]
    }

    pub fn push_stack(&mut self, var_count: u16, fn_no: u16, args: Vec<Variable>, info_no: usize) {
        if self.current_file().get_functions()[fn_no as usize].is_generator() {
            self.create_coroutine(fn_no, args);
        } else {
            let stack_height = self.variables.len();
            self.frames.push(StackFrame::new(
                var_count,
                fn_no,
                info_no,
                args,
                stack_height,
            ));
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
        if self.current_file().get_functions()[fn_no as usize].is_generator() {
            self.coroutine_from_frame(fn_no, args, frame);
        } else {
            let stack_height = self.variables.len();
            self.frames.push(StackFrame::from_old(
                var_count,
                fn_no,
                info_no,
                args,
                frame,
                stack_height,
            ));
        }
    }

    pub fn push_native(&mut self) {
        let stack_height = self.variables.len();
        self.frames.push(StackFrame::native(stack_height));
    }

    pub fn pop_native(&mut self) {
        debug_assert!(self.is_native());
        self.pop_stack();
    }

    pub fn pop_stack(&mut self) {
        if self.is_generator() {
            self.borrowed_iterators.pop();
        }
        let last_stack_frame = self
            .frames
            .pop()
            .expect("Frame stack should never be empty");
        for v in last_stack_frame.get_exceptions() {
            let last_frames = self.exception_frames.get_mut(v).expect(
                "In pop_stack(): popped frame has exception \
                    not covered in runtime's exception frames",
            );
            assert_eq!(last_frames.last().unwrap().1, self.frames.len() - 1);
            last_frames.pop();
            self.exception_stack.pop();
        }
        let stack_h = last_stack_frame.original_stack_height();
        if stack_h != 0 {
            let drain_end = self.variables.len() - self.ret_count;
            if drain_end < stack_h {
                self.frames.push(last_stack_frame);
                println!("{:#?}", self.variables);
                panic!(
                    "Attempted to remove a negative number of values ({}..{})\n{}",
                    stack_h,
                    drain_end,
                    self.frame_strings()
                )
            }
            self.variables.drain(stack_h..drain_end);
        }
    }

    pub fn is_native(&self) -> bool {
        self.last_frame().is_native()
    }

    pub fn is_bottom_stack(&self) -> bool {
        self.frames.len() == 1
    }

    pub fn get_fn_name(&self, file_no: usize, fn_no: u32) -> StringVar {
        self.files[file_no].get_functions()[fn_no as usize]
            .get_name()
            .to_string()
            .into()
    }

    pub fn throw(&mut self, exception: Variable) -> FnResult {
        let exc_type = exception.get_type();
        let frames = self.collect_stack_frames();
        let exc = InnerException::Std(exception, frames);
        self.unwind(exc_type, exc)
    }

    pub fn throw_quick<T: Into<StringVar>>(&mut self, exc_type: Type, message: T) -> FnResult {
        let frames = self.collect_stack_frames();
        let exc = InnerException::UnConstructed(exc_type, message.into(), frames);
        self.unwind(exc_type, exc)
    }

    pub fn throw_quick_native<T: Into<StringVar>, U>(
        &mut self,
        exc_type: Type,
        message: T,
    ) -> Result<U, ()> {
        assert!(
            self.is_native(),
            "throw_quick_native expected a native function\n{}",
            self.frame_strings()
        );
        let frames = self.collect_stack_frames();
        let exc = InnerException::UnConstructed(exc_type, message.into(), frames);
        self.thrown_exception = Option::Some(exc);
        Result::Err(())
    }

    pub fn resume_throw(&mut self) -> FnResult {
        match self.thrown_exception.take() {
            Option::Some(exception) => self.unwind(exception.get_type(), exception),
            Option::None => panic!(
                "resume_throw() called with no thrown exception\n{}",
                self.frame_strings()
            ),
        }
    }

    fn unwind(&mut self, exc_type: Type, exc: InnerException) -> FnResult {
        let frame = self.exception_frames.get(&exc_type.into());
        match frame.and_then(|vec| vec.last().cloned()) {
            Option::Some((location, frame_height)) => {
                self.unwind_to_height(location, frame_height, exc)
            }
            Option::None => self.unwind_to_empty(exc),
        }
    }

    pub fn do_static(&mut self) -> bool {
        let last_frame = self.last_frame();
        assert!(!last_frame.is_native());
        let triplet = (
            self.current_file_no(),
            last_frame.get_fn_number(),
            last_frame.current_pos(),
        );
        self.completed_statics.insert(triplet)
    }

    pub fn store_static(&mut self, index: usize, var: Variable) {
        self.static_vars
            .resize(max(self.static_vars.len(), index + 1), Variable::null());
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
        self.last_mut_frame()
            .add_exception_handler(exception_type.clone());
        self.exception_stack.push(exception_type);
    }

    pub fn remove_exception_handler(&mut self, exception_type: &Variable) {
        match self.exception_frames.get_mut(exception_type) {
            Option::Some(fr) => fr.pop(),
            Option::None => panic!(
                "Attempted to remove exception handler for {:?}: not found",
                exception_type
            ),
        };
        self.last_mut_frame()
            .remove_exception_handler(exception_type);
    }

    pub fn pop_handler(&mut self) {
        let val = self
            .exception_stack
            .pop()
            .expect("Called pop_handler with empty exception stack");
        self.remove_exception_handler(&val);
    }

    pub fn load_fn(&self, fn_no: u16) -> Variable {
        Rc::new(Lambda::new(
            self.current_file_no(),
            fn_no as u32,
            self.last_frame().clone(),
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

    pub fn return_n<const N: usize>(&mut self, var: [Variable; N]) -> FnResult {
        self.ret_count = N;
        self.variables.extend(var);
        FnResult::Ok(())
    }

    pub fn pop_return(&mut self) -> Variable {
        match replace(&mut self.ret_count, 0) {
            0 => panic!(
                "Attempted to call pop_return where no values were returned\n{}",
                self.frame_strings()
            ),
            1 => self.pop(),
            x => {
                let new_len = self.variables.len() - x + 1;
                self.variables.truncate(new_len);
                self.pop()
            }
        }
    }

    pub fn pop_returns(&mut self, ret_count: usize) -> Vec<Variable> {
        match replace(&mut self.ret_count, 0) {
            0 if ret_count == 0 => vec![],
            0 => panic!(
                "Attempted to call pop_returns where no values were returned\n{}",
                self.frame_strings()
            ),
            i => match i.cmp(&ret_count) {
                Ordering::Less => panic!(
                    "Runtime::pop_returns called with a count of {}, but only {} values were returned\n{}",
                    ret_count, i, self.frame_strings()
                ),
                Ordering::Equal => self
                    .variables
                    .drain(self.variables.len() - ret_count..)
                    .collect(),
                Ordering::Greater => {
                    let new_len = self.variables.len() - i + ret_count;
                    self.variables.truncate(new_len);
                    self.variables.drain(new_len - ret_count..).collect()
                }
            }
        }
    }

    pub fn pop_generator_returns(&mut self, ret_count: usize) -> Vec<Variable> {
        match self.ret_count {
            0 => panic!(
                "Attempted to return 0 values from generator\n{}",
                self.frame_strings()
            ),
            i => {
                assert!(ret_count > 0 && i > 0);
                self.pop_returns(min(i, ret_count))
            }
        }
    }

    pub fn pop_n(&mut self, count: usize) -> Vec<Variable> {
        let len = self.variables.len();
        self.variables.drain(len - count..).collect()
    }

    pub fn static_attr(&self, cls: &Type, name: Name) -> Option<Variable> {
        self.type_vars.get(cls).and_then(|x| x.get(name)).cloned()
    }

    pub fn set_static_attr(&mut self, cls: &Type, name: Name, var: Variable) {
        self.type_vars
            .entry(*cls)
            .or_insert_with(NameMap::new)
            .insert(name, var);
    }

    pub fn swap_2(&mut self) {
        let len = self.variables.len();
        self.variables.swap(len - 1, len - 2);
    }

    pub fn swap_n(&mut self, index: usize) {
        let value = self.variables.remove(self.variables.len() - index);
        self.variables.push(value);
    }

    pub fn swap_stack(&mut self, index_1: usize, index_2: usize) {
        let len = self.variables.len() - 1;
        self.variables.swap(len - index_1, len - index_2);
    }

    pub fn collect_stack_frames(&self) -> Vec<SFInfo> {
        self.frames.iter().map(StackFrame::exc_info).collect()
    }

    pub fn add_generator(&mut self, gen: Rc<Generator>) -> FnResult {
        match gen.take_frame() {
            Option::Some(mut val) => {
                val.set_stack_height(self.variables.len());
                self.frames.push(val)
            }
            Option::None => {
                return self.throw_quick(invalid_state(), "Generator already executing")
            }
        }
        self.variables.append(&mut gen.take_stack());
        self.borrowed_iterators.push(gen);
        FnResult::Ok(())
    }

    pub fn generator_end(&mut self) {
        debug_assert!(self.is_generator());
        let frame = self.frames.pop().unwrap();
        let old_height = frame.original_stack_height();
        let vec = self.variables.drain(old_height..).collect();
        self.variables.push(Option::None.into());
        self.set_ret(1);
        let gen = self
            .borrowed_iterators
            .pop()
            .expect("Yield called with no generator");
        gen.replace_vars(frame, vec);
    }

    pub fn generator_yield(&mut self, ret_count: usize) {
        debug_assert!(self.is_generator());
        let replace_start = self.variables.len() - ret_count;
        for x in &mut self.variables[replace_start..] {
            let old_x = take(x);
            *x = Option::Some(old_x).into()
        }
        self.set_ret(ret_count);
        let frame = self.frames.pop().unwrap();
        let old_height = frame.original_stack_height();
        let vec = self.variables.drain(old_height..replace_start).collect();
        let gen = self
            .borrowed_iterators
            .pop()
            .expect("Yield called with no generator");
        gen.replace_vars(frame, vec);
    }

    pub fn is_generator(&self) -> bool {
        self.current_file().get_functions()[self.last_frame().get_fn_number() as usize]
            .is_generator()
    }

    pub fn jump_table(&self, num: usize) -> &JumpTable {
        self.current_file().jump_table(num)
    }

    pub fn pop_err(&mut self) -> Result<Variable, ()> {
        self.thrown_exception
            .take()
            .expect("pop_err called with no thrown exception")
            .create(self)
    }

    pub fn pop_err_if(&mut self, t: Type) -> Result<Option<Variable>, ()> {
        Result::Ok(match &mut self.thrown_exception {
            Option::Some(exc) if exc.get_type() != t => Option::None,
            err @ Option::Some(_) => Option::Some(err.take().unwrap().create(self)?),
            Option::None => panic!("pop_err called with no thrown exception"),
        })
    }

    pub fn frame_strings(&self) -> String {
        frame_strings(self.frames.iter().rev().map(StackFrame::exc_info), self)
    }

    pub fn class_no(&self, val: u32) -> Type {
        self.current_file().get_constants()[val as usize]
            .clone()
            .into()
    }

    fn create_coroutine(&mut self, fn_no: u16, args: Vec<Variable>) {
        let stack_height = self.variables.len();
        let frame = StackFrame::new(0, fn_no, self.current_file_no(), args, stack_height);
        let stack = Vec::new();
        self.push(Rc::new(Generator::new(frame, stack)).into())
    }

    fn coroutine_from_frame(&mut self, fn_no: u16, args: Vec<Variable>, frame: StackFrame) {
        let stack_height = self.variables.len();
        let new_frame =
            StackFrame::from_old(0, fn_no, self.current_file_no(), args, frame, stack_height);
        let stack = Vec::new();
        self.push(Rc::new(Generator::new(new_frame, stack)).into())
    }

    fn unwind_to_height(
        &mut self,
        location: u32,
        frame_height: usize,
        exception: InnerException,
    ) -> FnResult {
        while self.frames.len() > frame_height {
            if self.is_native() {
                self.thrown_exception = Option::Some(exception);
                return FnResult::Err(());
            }
            self.pop_stack();
        }
        self.exception_frames
            .get_mut(&exception.get_type().into())
            .unwrap()
            .pop();
        self.remove_exception_handler(&exception.get_type().into());
        self.goto(location);
        FnResult::Ok(())
    }

    fn unwind_to_empty(&mut self, exception: InnerException) -> FnResult {
        let old_ret = self.ret_count;
        self.ret_count = 0;
        while !self.frames.is_empty() {
            if self.is_native() {
                self.thrown_exception = Option::Some(exception);
                return FnResult::Err(());
            }
            self.pop_stack();
        }
        self.ret_count = old_ret;
        panic!(
            "{}",
            exception
                .str(self)
                .expect("Exception.operator str should not throw an exception")
        );
    }

    pub fn test<F>(f: F) -> Result<Variable, ()>
    where
        F: FnOnce(&mut Runtime) -> FnResult,
    {
        let mut test_runtime = Self::new(vec![], 0);
        match f(&mut test_runtime) {
            Result::Ok(_) => Result::Ok(test_runtime.pop_return()),
            Result::Err(_) => Result::Err(()),
        }
    }
}

impl InnerException {
    fn get_type(&self) -> Type {
        match self {
            InnerException::Std(v, _) => v.get_type(),
            InnerException::UnConstructed(t, ..) => *t,
        }
    }

    fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let exc = self.deconstruct(runtime)?;
        Result::Ok(
            format!(
                "{}:\n{}\n{}",
                exc.cls.str(),
                exc.msg,
                frame_strings(exc.frames.into_iter().rev(), runtime)
            )
            .into(),
        )
    }

    fn deconstruct(self, runtime: &mut Runtime) -> Result<DeconstructedExc, ()> {
        match self {
            InnerException::Std(var, frames) => {
                let cls = var.get_type();
                var.index(Name::Attribute("msg"), runtime)?
                    .call((Vec::new(), runtime))?;
                let result = StringVar::from(runtime.pop_return());
                Result::Ok(DeconstructedExc::new(cls, result, frames))
            }
            InnerException::UnConstructed(cls, msg, frames) => {
                Result::Ok(DeconstructedExc::new(cls, msg, frames))
            }
        }
    }

    fn create(self, runtime: &mut Runtime) -> Result<Variable, ()> {
        Result::Ok(match self {
            InnerException::Std(e, _) => e,
            // FIXME: Won't collect stack frames properly
            InnerException::UnConstructed(t, s, _) => t.create_inst(vec![s.into()], runtime)?,
        })
    }
}

impl DeconstructedExc {
    pub fn new(cls: Type, msg: StringVar, frames: Vec<SFInfo>) -> DeconstructedExc {
        DeconstructedExc { cls, msg, frames }
    }
}
