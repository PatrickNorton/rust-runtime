use crate::custom_types::coroutine::Generator;
use crate::custom_types::exceptions::invalid_state;
use crate::custom_types::lambda::Lambda;
use crate::executor;
use crate::file_info::FileInfo;
use crate::function::NativeFunction;
use crate::jump_table::JumpTable;
use crate::method::{NativeCopyMethod, NativeMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::stack_frame::{SFInfo, StackFrame};
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::mem::take;
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
    type_vars: HashMap<Type, HashMap<Name, Variable>>,
    ret_count: usize,
    borrowed_iterators: Vec<Rc<Generator>>,
    thrown_exception: Option<InnerException>,

    files: Vec<FileInfo>,
}

#[derive(Debug)]
enum InnerException {
    Std(Variable),
    UnConstructed(Type, StringVar, Vec<SFInfo>),
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

    pub fn tail_quick(&mut self, fn_no: u16) {
        let stack_height = self.variables.len();
        let file_no = self.current_file_no();
        let frame = self.last_mut_frame();
        *frame = StackFrame::new(0, fn_no, file_no, Vec::new(), stack_height);
    }

    pub fn call_tos_or_goto(&mut self, argc: u16) -> FnResult {
        let args = self.load_args(argc as usize);
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
            if result.is_err() && !self.is_native() {
                self.resume_throw()
            } else {
                result
            }
        } else {
            func(this, args, self)
        }
    }

    pub fn call_copy_method<T>(
        &mut self,
        func: NativeCopyMethod<T>,
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
        self.frames.push(StackFrame::native());
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
                panic!(
                    "Attempted to remove a negative number of values ({}..{})\n{}",
                    stack_h,
                    drain_end,
                    self.stack_frames()
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
            .clone()
            .into()
    }

    pub fn throw(&mut self, exception: Variable) -> FnResult {
        let exc_type = exception.get_type();
        let exc = InnerException::Std(exception);
        self.unwind(exc_type, exc)
    }

    pub fn throw_quick(&mut self, exc_type: Type, message: StringVar) -> FnResult {
        let frames = self.collect_stack_frames();
        let exc = InnerException::UnConstructed(exc_type, message, frames);
        self.unwind(exc_type, exc)
    }

    fn unwind(&mut self, exc_type: Type, exc: InnerException) -> FnResult {
        let frame = self.exception_frames.get(&exc_type.into());
        match frame.and_then(|vec| vec.last()) {
            Option::Some(pair) => {
                let pair2 = *pair;
                self.unwind_to_height(pair2.0, pair2.1, exc)
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
            .resize(max(self.static_vars.len(), index + 1), Variable::default());
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

    pub fn pop_returns(&mut self, ret_count: usize) -> Vec<Variable> {
        match self.ret_count {
            0 if ret_count == 0 => vec![],
            0 => panic!("Attempted to call pop_returns where no values were returned"),
            i if i < ret_count => panic!(
                "Runtime::pop_returns called with a count of {}, but only {} values were returned",
                ret_count, i
            ),
            i if i == ret_count => self
                .variables
                .drain(self.variables.len() - ret_count..)
                .collect(),
            i => {
                let new_len = self.variables.len() - i + ret_count;
                self.ret_count = 0;
                self.variables.truncate(new_len);
                self.variables.drain(new_len - ret_count..).collect()
            }
        }
    }

    pub fn pop_n(&mut self, count: usize) -> Vec<Variable> {
        let len = self.variables.len();
        self.variables.drain(len - count..).collect()
    }

    pub fn static_attr(&self, cls: &Type, name: Name) -> Variable {
        self.type_vars[cls][&name].clone()
    }

    pub fn set_static_attr(&mut self, cls: &Type, name: Name, var: Variable) {
        match self.type_vars.get_mut(cls) {
            Option::Some(val) => {
                val.insert(name, var);
            }
            Option::None => {
                self.type_vars.insert(*cls, hash_map!(name => var));
            }
        };
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
            Option::Some(val) => self.frames.push(val),
            Option::None => {
                return self.throw_quick(invalid_state(), "Generator already executing".into())
            }
        }
        self.variables.append(&mut gen.take_stack());
        self.borrowed_iterators.push(gen);
        FnResult::Ok(())
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
        let vec = Vec::new(); // FIXME: Clear stack
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
        match self
            .thrown_exception
            .as_mut()
            .expect("pop_err called with no thrown exception")
        {
            InnerException::Std(val) => {
                if val.get_type() == t {
                    let result = Result::Ok(Option::Some(take(val)));
                    self.thrown_exception = Option::None;
                    result
                } else {
                    Result::Ok(Option::None)
                }
            }
            InnerException::UnConstructed(ty, s, _) => {
                if *ty == t {
                    let result =
                        Result::Ok(Option::Some(t.create_inst(vec![take(s).into()], self)?));
                    self.thrown_exception = Option::None;
                    result
                } else {
                    Result::Ok(Option::None)
                }
            }
        }
    }

    pub fn stack_frames(&self) -> String {
        let mut result = String::new();
        for frame in self.frames.iter().rev() {
            if !frame.is_native() {
                let file = &self.files[frame.file_no()];
                let fn_no = frame.get_fn_number();
                let fn_pos = frame.current_pos();
                let func = &file.get_functions()[fn_no as usize];
                let fn_name = func.get_name();
                result.push_str(&*format!(
                    "    at {}:{} ({})\n",
                    fn_name,
                    fn_pos,
                    file.get_name()
                ))
            }
        }
        result
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

    fn frame_strings(&self, frames: &[SFInfo]) -> String {
        frames
            .iter()
            .enumerate()
            .map(|f| self.frame_str(f.0, f.1))
            .collect()
    }

    fn frame_str(&self, no: usize, frame: &SFInfo) -> String {
        if frame.is_native() {
            format!("{}: [unknown native function]\n", no)
        } else {
            let file = &self.files[frame.file_no()];
            let file_name = file.get_name();
            let fn_name = file.get_functions()[frame.fn_no() as usize].get_name();
            format!("{}: {} {}\n", no, file_name, fn_name)
        }
    }

    pub fn resume_throw(&mut self) -> FnResult {
        let exception = self
            .thrown_exception
            .take()
            .expect("resume_throw() called with no thrown exception");
        match self
            .exception_frames
            .get(&exception.get_type().into())
            .and_then(|x| x.last().copied())
        {
            Option::Some((location, frame_height)) => {
                self.unwind_to_height(location, frame_height, exception)
            }
            Option::None => self.unwind_to_empty(exception),
        }
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
        exception.str(self).map(|x| panic!("{}", x))
    }
}

impl InnerException {
    fn get_type(&self) -> Type {
        match self {
            InnerException::Std(v) => v.get_type(),
            InnerException::UnConstructed(t, ..) => *t,
        }
    }

    fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            InnerException::Std(var) => var.str(runtime),
            InnerException::UnConstructed(_, msg, frames) => {
                Result::Ok(format!("{}\n{}", msg, runtime.frame_strings(&*frames)).into())
            }
        }
    }

    fn create(self, runtime: &mut Runtime) -> Result<Variable, ()> {
        Result::Ok(match self {
            InnerException::Std(e) => e,
            InnerException::UnConstructed(t, s, _) => t.create_inst(vec![s.into()], runtime)?, // FIXME: Won't collect stack frames properly
        })
    }
}
