use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::{Chunk, OpCode};
use crate::value::{AdtInstance, Closure, Function, NativeFunction, TypeConstructorDef, Upvalue, UpvalueLocation, Value};

#[derive(Debug, Clone)]
struct CallFrame {
    closure: Option<Rc<Closure>>,  // None for plain functions, Some for closures
    function: Rc<Function>,
    ip: usize,
    slots_start: usize, // Where this frame's locals start on the stack
}

pub struct VM {
    chunk: Chunk,              // The main/script chunk
    functions: Vec<Rc<Chunk>>, // Compiled function chunks (Rc for cheap cloning)
    frames: Vec<CallFrame>,    // Call stack
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    open_upvalues: Vec<Rc<RefCell<Upvalue>>>,  // Open upvalues pointing to stack
}

impl VM {
    pub fn new(chunk: Chunk, functions: Vec<Rc<Chunk>>) -> Self {
        let mut vm = VM {
            chunk,
            functions,
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            open_upvalues: Vec::new(),
        };
        vm.define_natives();
        vm
    }

    fn define_natives(&mut self) {
        // lengte(lys) - returns the length of a list or string
        self.globals.insert(
            String::from("lengte"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("lengte"),
                arity: 1,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => Ok(Value::Number(items.len() as f64)),
                        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
                        _ => Err("lengte() verwag 'n lys of string.".to_string()),
                    }
                },
            })),
        );

        // kop(lys) - returns the first element of a list
        self.globals.insert(
            String::from("kop"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("kop"),
                arity: 1,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => {
                            if items.is_empty() {
                                Err("Kan nie kop van leë lys kry nie.".to_string())
                            } else {
                                Ok(items[0].clone())
                            }
                        }
                        _ => Err("kop() verwag 'n lys.".to_string()),
                    }
                },
            })),
        );

        // stert(lys) - returns all but the first element of a list
        self.globals.insert(
            String::from("stert"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("stert"),
                arity: 1,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => {
                            if items.is_empty() {
                                Err("Kan nie stert van leë lys kry nie.".to_string())
                            } else {
                                let tail: Vec<Value> = items[1..].to_vec();
                                Ok(Value::List(Rc::new(tail)))
                            }
                        }
                        _ => Err("stert() verwag 'n lys.".to_string()),
                    }
                },
            })),
        );

        // leeg(lys) - returns true if list is empty
        self.globals.insert(
            String::from("leeg"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("leeg"),
                arity: 1,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => Ok(Value::Boolean(items.is_empty())),
                        Value::String(s) => Ok(Value::Boolean(s.is_empty())),
                        _ => Err("leeg() verwag 'n lys of string.".to_string()),
                    }
                },
            })),
        );

        // voeg_by(element, lys) - prepends element to list (cons)
        self.globals.insert(
            String::from("voeg_by"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("voeg_by"),
                arity: 2,
                func: |args| {
                    match &args[1] {
                        Value::List(items) => {
                            let mut new_list = vec![args[0].clone()];
                            new_list.extend(items.iter().cloned());
                            Ok(Value::List(Rc::new(new_list)))
                        }
                        _ => Err("voeg_by() verwag 'n lys as tweede argument.".to_string()),
                    }
                },
            })),
        );

        // heg_aan(lys, element) - appends element to list
        self.globals.insert(
            String::from("heg_aan"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("heg_aan"),
                arity: 2,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => {
                            let mut new_list = items.as_ref().clone();
                            new_list.push(args[1].clone());
                            Ok(Value::List(Rc::new(new_list)))
                        }
                        _ => Err("heg_aan() verwag 'n lys as eerste argument.".to_string()),
                    }
                },
            })),
        );

        // ketting(lys1, lys2) - concatenates two lists
        self.globals.insert(
            String::from("ketting"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("ketting"),
                arity: 2,
                func: |args| {
                    match (&args[0], &args[1]) {
                        (Value::List(a), Value::List(b)) => {
                            let mut new_list = a.as_ref().clone();
                            new_list.extend(b.iter().cloned());
                            Ok(Value::List(Rc::new(new_list)))
                        }
                        _ => Err("ketting() verwag twee lyste.".to_string()),
                    }
                },
            })),
        );

        // omgekeer(lys) - reverses a list
        self.globals.insert(
            String::from("omgekeer"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("omgekeer"),
                arity: 1,
                func: |args| {
                    match &args[0] {
                        Value::List(items) => {
                            let reversed: Vec<Value> = items.iter().rev().cloned().collect();
                            Ok(Value::List(Rc::new(reversed)))
                        }
                        _ => Err("omgekeer() verwag 'n lys.".to_string()),
                    }
                },
            })),
        );

        // Higher-order functions are handled specially in Call opcode
        // These are placeholder registrations so they're recognized as functions

        // kaart(lys, fn) - map function over list
        self.globals.insert(
            String::from("kaart"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("kaart"),
                arity: 2,
                func: |_| Err("kaart() moet spesiaal hanteer word.".to_string()),
            })),
        );

        // filter(lys, fn) - filter list by predicate
        self.globals.insert(
            String::from("filter"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("filter"),
                arity: 2,
                func: |_| Err("filter() moet spesiaal hanteer word.".to_string()),
            })),
        );

        // vou(lys, begin, fn) - fold/reduce list
        self.globals.insert(
            String::from("vou"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("vou"),
                arity: 3,
                func: |_| Err("vou() moet spesiaal hanteer word.".to_string()),
            })),
        );

        // vir_elk(lys, fn) - for each element, call function (returns nil)
        self.globals.insert(
            String::from("vir_elk"),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: String::from("vir_elk"),
                arity: 2,
                func: |_| Err("vir_elk() moet spesiaal hanteer word.".to_string()),
            })),
        );
    }

    pub fn run(&mut self) -> Result<(), String> {
        // Start executing the main chunk directly (not as a function call)
        self.run_chunk(&self.chunk.clone())
    }

    fn run_chunk(&mut self, chunk: &Chunk) -> Result<(), String> {
        let mut ip = 0;

        loop {
            if ip >= chunk.code.len() {
                return Ok(());
            }

            let instruction = &chunk.code[ip];
            ip += 1;

            match instruction {
                OpCode::Constant(idx) => {
                    let value = chunk.constants[*idx].clone();
                    self.push(value);
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::GetVar(name) | OpCode::GetGlobal(name) => {
                    let value = self
                        .globals
                        .get(name)
                        .cloned()
                        .ok_or_else(|| format!("Ongedefinieerde veranderlike: '{}'", name))?;
                    self.push(value);
                }
                OpCode::SetVar(name) | OpCode::SetGlobal(name) => {
                    let value = self.peek()?.clone();
                    if !self.globals.contains_key(name) {
                        return Err(format!("Ongedefinieerde veranderlike: '{}'", name));
                    }
                    self.globals.insert(name.clone(), value);
                }
                OpCode::DefineGlobal(name) => {
                    let value = self.pop()?;
                    self.globals.insert(name.clone(), value);
                }
                OpCode::GetLocal(slot) => {
                    let base = if self.frames.is_empty() {
                        0
                    } else {
                        self.frames.last().unwrap().slots_start
                    };
                    let value = self.stack[base + *slot].clone();
                    self.push(value);
                }
                OpCode::SetLocal(slot) => {
                    let base = if self.frames.is_empty() {
                        0
                    } else {
                        self.frames.last().unwrap().slots_start
                    };
                    let value = self.peek()?.clone();
                    self.stack[base + *slot] = value;
                }
                OpCode::GetUpvalue(slot) => {
                    if let Some(frame) = self.frames.last() {
                        if let Some(ref closure) = frame.closure {
                            let value = {
                                let upvalue = closure.upvalues[*slot].borrow();
                                match &upvalue.location {
                                    UpvalueLocation::Open(idx) => self.stack[*idx].clone(),
                                    UpvalueLocation::Closed(val) => val.clone(),
                                }
                            };
                            self.push(value);
                        } else {
                            return Err("GetUpvalue called on non-closure function".to_string());
                        }
                    } else {
                        return Err("GetUpvalue called outside of function".to_string());
                    }
                }
                OpCode::SetUpvalue(slot) => {
                    if let Some(frame) = self.frames.last() {
                        if let Some(ref closure) = frame.closure {
                            let value = self.peek()?.clone();
                            let mut upvalue = closure.upvalues[*slot].borrow_mut();
                            match &mut upvalue.location {
                                UpvalueLocation::Open(idx) => {
                                    self.stack[*idx] = value;
                                }
                                UpvalueLocation::Closed(val) => {
                                    *val = value;
                                }
                            }
                        } else {
                            return Err("SetUpvalue called on non-closure function".to_string());
                        }
                    } else {
                        return Err("SetUpvalue called outside of function".to_string());
                    }
                }
                OpCode::Closure(const_idx, upvalue_descs) => {
                    let value = chunk.constants[*const_idx].clone();
                    if let Value::Function(func) = value {
                        let base = if self.frames.is_empty() {
                            0
                        } else {
                            self.frames.last().unwrap().slots_start
                        };

                        let mut upvalues = Vec::new();
                        for desc in upvalue_descs {
                            let upvalue = if desc.is_local {
                                // Capture from stack
                                self.capture_upvalue(base + desc.index)
                            } else {
                                // Capture from enclosing closure's upvalue
                                if let Some(frame) = self.frames.last() {
                                    if let Some(ref closure) = frame.closure {
                                        Rc::clone(&closure.upvalues[desc.index])
                                    } else {
                                        return Err("Cannot capture upvalue from non-closure".to_string());
                                    }
                                } else {
                                    return Err("Cannot capture upvalue outside of function".to_string());
                                }
                            };
                            upvalues.push(upvalue);
                        }

                        let closure = Closure {
                            function: func,
                            upvalues,
                        };
                        self.push(Value::Closure(Rc::new(closure)));
                    } else {
                        return Err("Closure constant is not a function".to_string());
                    }
                }
                OpCode::CloseUpvalue => {
                    let top = self.stack.len() - 1;
                    self.close_upvalues(top);
                    self.pop()?;
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x + y));
                        }
                        (Value::String(x), Value::String(y)) => {
                            let result = format!("{}{}", x, y);
                            self.push(Value::String(Rc::new(result)));
                        }
                        (Value::String(x), _) => {
                            let result = format!("{}{}", x, b);
                            self.push(Value::String(Rc::new(result)));
                        }
                        (_, Value::String(y)) => {
                            let result = format!("{}{}", a, y);
                            self.push(Value::String(Rc::new(result)));
                        }
                        _ => return Err("Operande moet nommers of stringe wees vir '+'.".to_string()),
                    }
                }
                OpCode::Subtract => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x - y));
                        }
                        _ => return Err("Operande moet nommers wees vir '-'.".to_string()),
                    }
                }
                OpCode::Multiply => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x * y));
                        }
                        _ => return Err("Operande moet nommers wees vir '*'.".to_string()),
                    }
                }
                OpCode::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            if y == 0.0 {
                                return Err("Deling deur nul.".to_string());
                            }
                            self.push(Value::Number(x / y));
                        }
                        _ => return Err("Operande moet nommers wees vir '/'.".to_string()),
                    }
                }
                OpCode::Modulo => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            if y == 0.0 {
                                return Err("Modulo deur nul.".to_string());
                            }
                            self.push(Value::Number(x % y));
                        }
                        _ => return Err("Operande moet nommers wees vir '%'.".to_string()),
                    }
                }
                OpCode::Negate => {
                    let value = self.pop()?;
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => return Err("Operand moet 'n nommer wees vir negasie.".to_string()),
                    }
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(self.values_equal(&a, &b)));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(!self.values_equal(&a, &b)));
                }
                OpCode::Less => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x < y));
                        }
                        _ => return Err("Operande moet nommers wees vir '<'.".to_string()),
                    }
                }
                OpCode::LessEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x <= y));
                        }
                        _ => return Err("Operande moet nommers wees vir '<='.".to_string()),
                    }
                }
                OpCode::Greater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x > y));
                        }
                        _ => return Err("Operande moet nommers wees vir '>'.".to_string()),
                    }
                }
                OpCode::GreaterEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x >= y));
                        }
                        _ => return Err("Operande moet nommers wees vir '>='.".to_string()),
                    }
                }
                OpCode::Not => {
                    let value = self.pop()?;
                    self.push(Value::Boolean(!value.is_truthy()));
                }
                OpCode::And | OpCode::Or => {
                    // Handled by jump instructions
                }
                OpCode::Print => {
                    let value = self.pop()?;
                    println!("{}", value);
                }
                OpCode::Jump(target) => {
                    ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let condition = self.peek()?;
                    if !condition.is_truthy() {
                        ip = *target;
                    }
                }
                OpCode::Call(arg_count) => {
                    let callee_idx = self.stack.len() - *arg_count - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Function(func) => {
                            if *arg_count != func.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    func.arity, arg_count
                                ));
                            }

                            // Get the function's chunk
                            let func_chunk = self.functions[func.chunk_index].clone();

                            // Push a new call frame
                            self.frames.push(CallFrame {
                                closure: None,
                                function: func.clone(),
                                ip,
                                slots_start: callee_idx,
                            });

                            // Execute the function
                            let result = self.run_function(func_chunk.clone(), callee_idx, None)?;

                            // Pop the call frame
                            let frame = self.frames.pop().unwrap();

                            // Restore IP
                            ip = frame.ip;

                            // Pop arguments and callee, push result
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Closure(closure) => {
                            if *arg_count != closure.function.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    closure.function.arity, arg_count
                                ));
                            }

                            // Get the function's chunk
                            let func_chunk = self.functions[closure.function.chunk_index].clone();

                            // Push a new call frame with the closure
                            self.frames.push(CallFrame {
                                closure: Some(Rc::clone(&closure)),
                                function: Rc::clone(&closure.function),
                                ip,
                                slots_start: callee_idx,
                            });

                            // Execute the function with closure context
                            let result = self.run_function(func_chunk.clone(), callee_idx, Some(Rc::clone(&closure)))?;

                            // Pop the call frame
                            let frame = self.frames.pop().unwrap();

                            // Restore IP
                            ip = frame.ip;

                            // Pop arguments and callee, push result
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::NativeFunction(nf) => {
                            if *arg_count != nf.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    nf.arity, arg_count
                                ));
                            }

                            let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();

                            // Handle higher-order functions specially
                            let result = match nf.name.as_str() {
                                "kaart" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_kaart(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("kaart() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "filter" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_filter(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("filter() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "vou" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_vou(Rc::clone(list), args[1].clone(), args[2].clone())?,
                                        _ => return Err("vou() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "vir_elk" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_vir_elk(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("vir_elk() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                _ => (nf.func)(&args)?,
                            };

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::TypeConstructor(tc) => {
                            // Check arity
                            if *arg_count != tc.arity {
                                return Err(format!(
                                    "Konstruktor '{}' verwag {} argumente maar het {} ontvang.",
                                    tc.constructor_name, tc.arity, arg_count
                                ));
                            }

                            // For unit constructors (arity 0), they're already values themselves
                            // For constructors with fields, create an AdtInstance
                            let result = if tc.arity == 0 {
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields: Vec::new(),
                                }))
                            } else {
                                let fields: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields,
                                }))
                            };

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        _ => {
                            return Err("Kan slegs funksies oproep.".to_string());
                        }
                    }
                }
                OpCode::Return => {
                    // Return from main chunk
                    return Ok(());
                }
                OpCode::MakeList(count) => {
                    let start = self.stack.len() - *count;
                    let elements: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::List(Rc::new(elements)));
                }
                OpCode::GetIndex => {
                    let index = self.pop()?;
                    let list = self.pop()?;

                    match (list, index) {
                        (Value::List(items), Value::Number(n)) => {
                            let idx = n as i64;
                            let len = items.len() as i64;
                            // Support negative indexing
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err(format!(
                                    "Lys indeks buite perke: {} (lengte {})",
                                    idx, len
                                ));
                            }
                            self.push(items[actual_idx as usize].clone());
                        }
                        (Value::String(s), Value::Number(n)) => {
                            let idx = n as i64;
                            let len = s.chars().count() as i64;
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err(format!(
                                    "String indeks buite perke: {} (lengte {})",
                                    idx, len
                                ));
                            }
                            let ch: String = s.chars().nth(actual_idx as usize).unwrap().to_string();
                            self.push(Value::String(Rc::new(ch)));
                        }
                        _ => {
                            return Err("Kan slegs lyste en stringe indekseer.".to_string());
                        }
                    }
                }
                OpCode::CheckConstructor(name, arity) => {
                    let value = self.peek()?;
                    let matches = match value {
                        Value::Adt(adt) => {
                            adt.constructor_name == *name && adt.fields.len() == *arity
                        }
                        // Unit constructors might be TypeConstructor values
                        Value::TypeConstructor(tc) => {
                            tc.constructor_name == *name && tc.arity == *arity && *arity == 0
                        }
                        _ => false,
                    };
                    self.push(Value::Boolean(matches));
                }
                OpCode::GetField(index) => {
                    let value = self.peek()?;
                    match value {
                        Value::Adt(adt) => {
                            if *index < adt.fields.len() {
                                let field_value = adt.fields[*index].clone();
                                self.push(field_value);
                            } else {
                                return Err(format!(
                                    "Veld indeks {} buite perke vir konstruktor '{}' met {} velde.",
                                    index, adt.constructor_name, adt.fields.len()
                                ));
                            }
                        }
                        _ => {
                            return Err("Kan slegs velde van ADT-waardes kry.".to_string());
                        }
                    }
                }
                OpCode::Dup => {
                    let value = self.peek()?.clone();
                    self.push(value);
                }
                OpCode::GetFieldPop(index) => {
                    let value = self.pop()?;
                    match value {
                        Value::Adt(adt) => {
                            if *index < adt.fields.len() {
                                self.push(adt.fields[*index].clone());
                            } else {
                                return Err(format!(
                                    "Veld indeks {} buite perke vir konstruktor '{}' met {} velde.",
                                    index, adt.constructor_name, adt.fields.len()
                                ));
                            }
                        }
                        _ => {
                            return Err("Kan slegs velde van ADT-waardes kry.".to_string());
                        }
                    }
                }
                OpCode::TailCall(_) => {
                    // TailCall should never appear in the main script chunk
                    return Err("TailCall kan nie in die hoofskrip gebruik word nie.".to_string());
                }
            }
        }
    }

    fn run_function(&mut self, chunk: Rc<Chunk>, slots_start: usize, closure: Option<Rc<Closure>>) -> Result<Value, String> {
        // Use mutable variables to support tail call optimization
        let mut current_chunk = chunk;
        let mut current_slots_start = slots_start;
        let mut current_closure = closure;
        let mut ip = 0;

        loop {
            if ip >= current_chunk.code.len() {
                return Ok(Value::Nil);
            }

            let instruction = &current_chunk.code[ip];
            ip += 1;

            match instruction {
                OpCode::Constant(idx) => {
                    let value = current_chunk.constants[*idx].clone();
                    self.push(value);
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::GetVar(name) | OpCode::GetGlobal(name) => {
                    let value = self
                        .globals
                        .get(name)
                        .cloned()
                        .ok_or_else(|| format!("Ongedefinieerde veranderlike: '{}'", name))?;
                    self.push(value);
                }
                OpCode::SetVar(name) | OpCode::SetGlobal(name) => {
                    let value = self.peek()?.clone();
                    if !self.globals.contains_key(name) {
                        return Err(format!("Ongedefinieerde veranderlike: '{}'", name));
                    }
                    self.globals.insert(name.clone(), value);
                }
                OpCode::DefineGlobal(name) => {
                    let value = self.pop()?;
                    self.globals.insert(name.clone(), value);
                }
                OpCode::GetLocal(slot) => {
                    let value = self.stack[current_slots_start + *slot].clone();
                    self.push(value);
                }
                OpCode::SetLocal(slot) => {
                    let value = self.peek()?.clone();
                    self.stack[current_slots_start + *slot] = value;
                }
                OpCode::GetUpvalue(slot) => {
                    if let Some(ref cl) = current_closure {
                        let value = {
                            let upvalue = cl.upvalues[*slot].borrow();
                            match &upvalue.location {
                                UpvalueLocation::Open(idx) => self.stack[*idx].clone(),
                                UpvalueLocation::Closed(val) => val.clone(),
                            }
                        };
                        self.push(value);
                    } else {
                        return Err("GetUpvalue called on non-closure function".to_string());
                    }
                }
                OpCode::SetUpvalue(slot) => {
                    if let Some(ref cl) = current_closure {
                        let value = self.peek()?.clone();
                        let mut upvalue = cl.upvalues[*slot].borrow_mut();
                        match &mut upvalue.location {
                            UpvalueLocation::Open(idx) => {
                                self.stack[*idx] = value;
                            }
                            UpvalueLocation::Closed(val) => {
                                *val = value;
                            }
                        }
                    } else {
                        return Err("SetUpvalue called on non-closure function".to_string());
                    }
                }
                OpCode::Closure(const_idx, upvalue_descs) => {
                    let value = current_chunk.constants[*const_idx].clone();
                    if let Value::Function(func) = value {
                        let mut upvalues = Vec::new();
                        for desc in upvalue_descs {
                            let upvalue = if desc.is_local {
                                // Capture from stack
                                self.capture_upvalue(current_slots_start + desc.index)
                            } else {
                                // Capture from enclosing closure's upvalue
                                if let Some(ref cl) = current_closure {
                                    Rc::clone(&cl.upvalues[desc.index])
                                } else {
                                    return Err("Cannot capture upvalue from non-closure".to_string());
                                }
                            };
                            upvalues.push(upvalue);
                        }

                        let new_closure = Closure {
                            function: func,
                            upvalues,
                        };
                        self.push(Value::Closure(Rc::new(new_closure)));
                    } else {
                        return Err("Closure constant is not a function".to_string());
                    }
                }
                OpCode::CloseUpvalue => {
                    let top = self.stack.len() - 1;
                    self.close_upvalues(top);
                    self.pop()?;
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x + y));
                        }
                        (Value::String(x), Value::String(y)) => {
                            let result = format!("{}{}", x, y);
                            self.push(Value::String(Rc::new(result)));
                        }
                        (Value::String(x), _) => {
                            let result = format!("{}{}", x, b);
                            self.push(Value::String(Rc::new(result)));
                        }
                        (_, Value::String(y)) => {
                            let result = format!("{}{}", a, y);
                            self.push(Value::String(Rc::new(result)));
                        }
                        _ => return Err("Operande moet nommers of stringe wees vir '+'.".to_string()),
                    }
                }
                OpCode::Subtract => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x - y));
                        }
                        _ => return Err("Operande moet nommers wees vir '-'.".to_string()),
                    }
                }
                OpCode::Multiply => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x * y));
                        }
                        _ => return Err("Operande moet nommers wees vir '*'.".to_string()),
                    }
                }
                OpCode::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            if y == 0.0 {
                                return Err("Deling deur nul.".to_string());
                            }
                            self.push(Value::Number(x / y));
                        }
                        _ => return Err("Operande moet nommers wees vir '/'.".to_string()),
                    }
                }
                OpCode::Modulo => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            if y == 0.0 {
                                return Err("Modulo deur nul.".to_string());
                            }
                            self.push(Value::Number(x % y));
                        }
                        _ => return Err("Operande moet nommers wees vir '%'.".to_string()),
                    }
                }
                OpCode::Negate => {
                    let value = self.pop()?;
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => return Err("Operand moet 'n nommer wees vir negasie.".to_string()),
                    }
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(self.values_equal(&a, &b)));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(!self.values_equal(&a, &b)));
                }
                OpCode::Less => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x < y));
                        }
                        _ => return Err("Operande moet nommers wees vir '<'.".to_string()),
                    }
                }
                OpCode::LessEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x <= y));
                        }
                        _ => return Err("Operande moet nommers wees vir '<='.".to_string()),
                    }
                }
                OpCode::Greater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x > y));
                        }
                        _ => return Err("Operande moet nommers wees vir '>'.".to_string()),
                    }
                }
                OpCode::GreaterEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Boolean(x >= y));
                        }
                        _ => return Err("Operande moet nommers wees vir '>='.".to_string()),
                    }
                }
                OpCode::Not => {
                    let value = self.pop()?;
                    self.push(Value::Boolean(!value.is_truthy()));
                }
                OpCode::And | OpCode::Or => {
                    // Handled by jump instructions
                }
                OpCode::Print => {
                    let value = self.pop()?;
                    println!("{}", value);
                }
                OpCode::Jump(target) => {
                    ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let condition = self.peek()?;
                    if !condition.is_truthy() {
                        ip = *target;
                    }
                }
                OpCode::Call(arg_count) => {
                    let callee_idx = self.stack.len() - *arg_count - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Function(func) => {
                            if *arg_count != func.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    func.arity, arg_count
                                ));
                            }

                            let func_chunk = self.functions[func.chunk_index].clone();

                            self.frames.push(CallFrame {
                                closure: None,
                                function: func.clone(),
                                ip,
                                slots_start: callee_idx,
                            });

                            let result = self.run_function(func_chunk.clone(), callee_idx, None)?;

                            let frame = self.frames.pop().unwrap();
                            ip = frame.ip;

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Closure(cl) => {
                            if *arg_count != cl.function.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    cl.function.arity, arg_count
                                ));
                            }

                            let func_chunk = self.functions[cl.function.chunk_index].clone();

                            self.frames.push(CallFrame {
                                closure: Some(Rc::clone(&cl)),
                                function: Rc::clone(&cl.function),
                                ip,
                                slots_start: callee_idx,
                            });

                            let result = self.run_function(func_chunk.clone(), callee_idx, Some(Rc::clone(&cl)))?;

                            let frame = self.frames.pop().unwrap();
                            ip = frame.ip;

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::NativeFunction(nf) => {
                            if *arg_count != nf.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    nf.arity, arg_count
                                ));
                            }

                            let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();

                            // Handle higher-order functions specially
                            let result = match nf.name.as_str() {
                                "kaart" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_kaart(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("kaart() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "filter" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_filter(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("filter() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "vou" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_vou(Rc::clone(list), args[1].clone(), args[2].clone())?,
                                        _ => return Err("vou() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                "vir_elk" => {
                                    match &args[0] {
                                        Value::List(list) => self.hof_vir_elk(Rc::clone(list), args[1].clone())?,
                                        _ => return Err("vir_elk() verwag 'n lys as eerste argument.".to_string()),
                                    }
                                }
                                _ => (nf.func)(&args)?,
                            };

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::TypeConstructor(tc) => {
                            // Check arity
                            if *arg_count != tc.arity {
                                return Err(format!(
                                    "Konstruktor '{}' verwag {} argumente maar het {} ontvang.",
                                    tc.constructor_name, tc.arity, arg_count
                                ));
                            }

                            // For unit constructors (arity 0), they're already values themselves
                            // For constructors with fields, create an AdtInstance
                            let result = if tc.arity == 0 {
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields: Vec::new(),
                                }))
                            } else {
                                let fields: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields,
                                }))
                            };

                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        _ => {
                            return Err("Kan slegs funksies oproep.".to_string());
                        }
                    }
                }
                OpCode::Return => {
                    // Get return value
                    let result = self.pop()?;

                    // Close all upvalues for locals being removed
                    self.close_upvalues(current_slots_start);

                    // Clean up local variables
                    self.stack.truncate(current_slots_start);

                    return Ok(result);
                }
                OpCode::TailCall(arg_count) => {
                    // Tail call optimization: reuse the current stack frame
                    let callee_idx = self.stack.len() - *arg_count - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Function(func) => {
                            if *arg_count != func.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    func.arity, arg_count
                                ));
                            }

                            // Close upvalues for current locals
                            self.close_upvalues(current_slots_start);

                            // Move arguments to current frame's slots
                            // Stack: [old_locals..., callee, arg1, arg2, ...]
                            // We want: [callee, arg1, arg2, ...]
                            let args: Vec<Value> = self.stack[callee_idx..].to_vec();
                            self.stack.truncate(current_slots_start);
                            for arg in args {
                                self.push(arg);
                            }

                            // Update chunk and reset IP
                            current_chunk = self.functions[func.chunk_index].clone();
                            current_closure = None;
                            ip = 0;
                            // Continue the loop with the new function
                        }
                        Value::Closure(cl) => {
                            if *arg_count != cl.function.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    cl.function.arity, arg_count
                                ));
                            }

                            // Close upvalues for current locals
                            self.close_upvalues(current_slots_start);

                            // Move arguments to current frame's slots
                            let args: Vec<Value> = self.stack[callee_idx..].to_vec();
                            self.stack.truncate(current_slots_start);
                            for arg in args {
                                self.push(arg);
                            }

                            // Update chunk, closure, and reset IP
                            current_chunk = self.functions[cl.function.chunk_index].clone();
                            current_closure = Some(Rc::clone(&cl));
                            ip = 0;
                            // Continue the loop with the new function
                        }
                        Value::NativeFunction(nf) => {
                            // Native functions can't be tail-called in the same way,
                            // just call them and return the result
                            if *arg_count != nf.arity {
                                return Err(format!(
                                    "Verwag {} argumente maar het {} ontvang.",
                                    nf.arity, arg_count
                                ));
                            }

                            let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                            let result = (nf.func)(&args)?;

                            self.close_upvalues(current_slots_start);
                            self.stack.truncate(current_slots_start);
                            return Ok(result);
                        }
                        Value::TypeConstructor(tc) => {
                            // Type constructors just create a value and return it
                            if *arg_count != tc.arity {
                                return Err(format!(
                                    "Konstruktor '{}' verwag {} argumente maar het {} ontvang.",
                                    tc.constructor_name, tc.arity, arg_count
                                ));
                            }

                            let result = if tc.arity == 0 {
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields: Vec::new(),
                                }))
                            } else {
                                let fields: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                                Value::Adt(Rc::new(AdtInstance {
                                    type_name: tc.type_name.clone(),
                                    constructor_name: tc.constructor_name.clone(),
                                    fields,
                                }))
                            };

                            self.close_upvalues(current_slots_start);
                            self.stack.truncate(current_slots_start);
                            return Ok(result);
                        }
                        _ => {
                            return Err("Kan slegs funksies oproep.".to_string());
                        }
                    }
                }
                OpCode::MakeList(count) => {
                    let start = self.stack.len() - *count;
                    let elements: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::List(Rc::new(elements)));
                }
                OpCode::GetIndex => {
                    let index = self.pop()?;
                    let list = self.pop()?;

                    match (list, index) {
                        (Value::List(items), Value::Number(n)) => {
                            let idx = n as i64;
                            let len = items.len() as i64;
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err(format!(
                                    "Lys indeks buite perke: {} (lengte {})",
                                    idx, len
                                ));
                            }
                            self.push(items[actual_idx as usize].clone());
                        }
                        (Value::String(s), Value::Number(n)) => {
                            let idx = n as i64;
                            let len = s.chars().count() as i64;
                            let actual_idx = if idx < 0 { len + idx } else { idx };
                            if actual_idx < 0 || actual_idx >= len {
                                return Err(format!(
                                    "String indeks buite perke: {} (lengte {})",
                                    idx, len
                                ));
                            }
                            let ch: String = s.chars().nth(actual_idx as usize).unwrap().to_string();
                            self.push(Value::String(Rc::new(ch)));
                        }
                        _ => {
                            return Err("Kan slegs lyste en stringe indekseer.".to_string());
                        }
                    }
                }
                OpCode::CheckConstructor(name, arity) => {
                    let value = self.peek()?;
                    let matches = match value {
                        Value::Adt(adt) => {
                            adt.constructor_name == *name && adt.fields.len() == *arity
                        }
                        Value::TypeConstructor(tc) => {
                            tc.constructor_name == *name && tc.arity == *arity && *arity == 0
                        }
                        _ => false,
                    };
                    self.push(Value::Boolean(matches));
                }
                OpCode::GetField(index) => {
                    let value = self.peek()?;
                    match value {
                        Value::Adt(adt) => {
                            if *index < adt.fields.len() {
                                let field_value = adt.fields[*index].clone();
                                self.push(field_value);
                            } else {
                                return Err(format!(
                                    "Veld indeks {} buite perke vir konstruktor '{}' met {} velde.",
                                    index, adt.constructor_name, adt.fields.len()
                                ));
                            }
                        }
                        _ => {
                            return Err("Kan slegs velde van ADT-waardes kry.".to_string());
                        }
                    }
                }
                OpCode::Dup => {
                    let value = self.peek()?.clone();
                    self.push(value);
                }
                OpCode::GetFieldPop(index) => {
                    let value = self.pop()?;
                    match value {
                        Value::Adt(adt) => {
                            if *index < adt.fields.len() {
                                self.push(adt.fields[*index].clone());
                            } else {
                                return Err(format!(
                                    "Veld indeks {} buite perke vir konstruktor '{}' met {} velde.",
                                    index, adt.constructor_name, adt.fields.len()
                                ));
                            }
                        }
                        _ => {
                            return Err("Kan slegs velde van ADT-waardes kry.".to_string());
                        }
                    }
                }
            }
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "Stapel onderloop.".to_string())
    }

    fn peek(&self) -> Result<&Value, String> {
        self.stack.last().ok_or_else(|| "Stapel is leeg.".to_string())
    }

    fn capture_upvalue(&mut self, stack_index: usize) -> Rc<RefCell<Upvalue>> {
        // Check if we already have an open upvalue for this stack slot
        for upvalue in &self.open_upvalues {
            if let UpvalueLocation::Open(idx) = upvalue.borrow().location {
                if idx == stack_index {
                    return Rc::clone(upvalue);
                }
            }
        }

        // Create new open upvalue
        let upvalue = Rc::new(RefCell::new(Upvalue {
            location: UpvalueLocation::Open(stack_index),
        }));
        self.open_upvalues.push(Rc::clone(&upvalue));
        upvalue
    }

    fn close_upvalues(&mut self, last: usize) {
        // Close all upvalues pointing at or above 'last' on the stack
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let should_close = {
                let upvalue = self.open_upvalues[i].borrow();
                if let UpvalueLocation::Open(idx) = upvalue.location {
                    idx >= last
                } else {
                    false
                }
            };

            if should_close {
                let stack_idx = {
                    let upvalue = self.open_upvalues[i].borrow();
                    if let UpvalueLocation::Open(idx) = upvalue.location {
                        idx
                    } else {
                        unreachable!()
                    }
                };
                let value = self.stack[stack_idx].clone();
                self.open_upvalues[i].borrow_mut().location = UpvalueLocation::Closed(value);
                self.open_upvalues.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Nil, Value::Nil) => true,
            (Value::List(x), Value::List(y)) => x == y,
            (Value::Function(x), Value::Function(y)) => Rc::ptr_eq(x, y),
            (Value::Closure(x), Value::Closure(y)) => Rc::ptr_eq(x, y),
            (Value::NativeFunction(x), Value::NativeFunction(y)) => Rc::ptr_eq(x, y),
            (Value::TypeConstructor(x), Value::TypeConstructor(y)) => Rc::ptr_eq(x, y),
            (Value::Adt(x), Value::Adt(y)) => {
                x.type_name == y.type_name
                    && x.constructor_name == y.constructor_name
                    && x.fields.len() == y.fields.len()
                    && x.fields.iter().zip(y.fields.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }

    /// Call a callable value with given arguments
    fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, String> {
        match callee {
            Value::Function(func) => {
                if args.len() != func.arity {
                    return Err(format!(
                        "Verwag {} argumente maar het {} ontvang.",
                        func.arity, args.len()
                    ));
                }

                // Set up the call
                let callee_idx = self.stack.len();
                self.push(Value::Function(Rc::clone(&func)));
                for arg in args {
                    self.push(arg);
                }

                let func_chunk = self.functions[func.chunk_index].clone();

                self.frames.push(CallFrame {
                    closure: None,
                    function: func.clone(),
                    ip: 0,
                    slots_start: callee_idx,
                });

                let result = self.run_function(func_chunk.clone(), callee_idx, None)?;

                self.frames.pop();
                self.stack.truncate(callee_idx);

                Ok(result)
            }
            Value::Closure(closure) => {
                if args.len() != closure.function.arity {
                    return Err(format!(
                        "Verwag {} argumente maar het {} ontvang.",
                        closure.function.arity, args.len()
                    ));
                }

                // Set up the call
                let callee_idx = self.stack.len();
                self.push(Value::Closure(Rc::clone(&closure)));
                for arg in args {
                    self.push(arg);
                }

                let func_chunk = self.functions[closure.function.chunk_index].clone();

                self.frames.push(CallFrame {
                    closure: Some(Rc::clone(&closure)),
                    function: Rc::clone(&closure.function),
                    ip: 0,
                    slots_start: callee_idx,
                });

                let result = self.run_function(func_chunk.clone(), callee_idx, Some(Rc::clone(&closure)))?;

                self.frames.pop();
                self.stack.truncate(callee_idx);

                Ok(result)
            }
            Value::NativeFunction(nf) => {
                if args.len() != nf.arity {
                    return Err(format!(
                        "Verwag {} argumente maar het {} ontvang.",
                        nf.arity, args.len()
                    ));
                }
                (nf.func)(&args)
            }
            Value::TypeConstructor(tc) => {
                if args.len() != tc.arity {
                    return Err(format!(
                        "Konstruktor '{}' verwag {} argumente maar het {} ontvang.",
                        tc.constructor_name, tc.arity, args.len()
                    ));
                }

                Ok(Value::Adt(Rc::new(AdtInstance {
                    type_name: tc.type_name.clone(),
                    constructor_name: tc.constructor_name.clone(),
                    fields: args,
                })))
            }
            _ => Err("Kan slegs funksies oproep.".to_string()),
        }
    }

    /// Higher-order function: kaart (map)
    fn hof_kaart(&mut self, list: Rc<Vec<Value>>, func: Value) -> Result<Value, String> {
        let mut results = Vec::with_capacity(list.len());
        for item in list.iter() {
            let result = self.call_value(func.clone(), vec![item.clone()])?;
            results.push(result);
        }
        Ok(Value::List(Rc::new(results)))
    }

    /// Higher-order function: filter
    fn hof_filter(&mut self, list: Rc<Vec<Value>>, func: Value) -> Result<Value, String> {
        let mut results = Vec::new();
        for item in list.iter() {
            let result = self.call_value(func.clone(), vec![item.clone()])?;
            if result.is_truthy() {
                results.push(item.clone());
            }
        }
        Ok(Value::List(Rc::new(results)))
    }

    /// Higher-order function: vou (fold/reduce)
    fn hof_vou(&mut self, list: Rc<Vec<Value>>, initial: Value, func: Value) -> Result<Value, String> {
        let mut acc = initial;
        for item in list.iter() {
            acc = self.call_value(func.clone(), vec![acc, item.clone()])?;
        }
        Ok(acc)
    }

    /// Higher-order function: vir_elk (for each)
    fn hof_vir_elk(&mut self, list: Rc<Vec<Value>>, func: Value) -> Result<Value, String> {
        for item in list.iter() {
            self.call_value(func.clone(), vec![item.clone()])?;
        }
        Ok(Value::Nil)
    }
}
