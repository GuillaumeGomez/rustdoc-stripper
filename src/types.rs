// Copyright 2016 Gomez Guillaume
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::{Debug, Display, Error, Formatter};
use std::borrow::Borrow;

pub const OUTPUT_COMMENT_FILE : &'static str = "comments.cmts";

pub struct ParseResult {
    pub event_list: Vec<EventInfo>,
    pub comment_lines: Vec<usize>,
    pub original_content : Vec<String>,
}

pub struct EventInfo {
    pub line: usize,
    pub event: EventType,
}

impl EventInfo {
    pub fn new(line: usize, event: EventType) -> EventInfo {
        EventInfo {
            line: line,
            event: event,
        }
    }
}

impl Debug for EventInfo {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self.event {
            EventType::Type(ref t) => { writeln!(fmt, "{:?}->{:?}", self.line, t) }
            _ => Ok(())
        }
    }
}

pub enum EventType {
    Comment(String),
    FileComment(String),
    Type(TypeStruct),
    InScope,
    OutScope,
}

impl Debug for EventType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            &EventType::Type(ref t) => writeln!(fmt, "{:?}", t),
            _ => Ok(())
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TypeStruct {
    pub ty: Type,
    pub parent: Option<Box<TypeStruct>>,
    pub name: String,
    pub args: Vec<String>,
}

impl TypeStruct {
    pub fn new(ty: Type, name: &str) -> TypeStruct {
        TypeStruct {
            ty: ty,
            name: name.to_owned(),
            args: vec!(),
            parent: None,
        }
    }

    /*pub fn from_args(ty: Type, args: Vec<String>) -> TypeStruct {
        TypeStruct {
            ty: ty,
            name: String::new(),
            args: args,
            parent: None,
        }
    }*/

    pub fn empty() -> TypeStruct {
        TypeStruct {
            ty: Type::Unknown,
            name: String::new(),
            args: Vec::new(),
            parent: None,
        }
    }

    pub fn get_depth(&self, ignore_macros: bool) -> usize {
        fn recur(ty: &Option<Box<TypeStruct>>, is_parent: bool, ignore_macros: bool) -> usize {
            match ty {
                &Some(ref t) => {
                    if ignore_macros && is_parent && t.ty == Type::Macro {
                        recur(&t.parent, true, ignore_macros)
                    } else {
                        recur(&t.parent, true, ignore_macros) + 1
                    }
                },
                _ => 0,
            }
        }
        recur(&self.parent, false, ignore_macros)
    }
}

impl Debug for TypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let parent = &self.parent;
        match parent {
            &Some(ref p) => write!(f, "{:?}§{} {}{}", p, self.ty, self.name, self.args.join(" ")),
            _ => write!(f, "{} {}{}", self.ty, self.name, self.args.join(" ")),
        }
    }
}

fn show(f: &mut Formatter, t: &TypeStruct, is_parent: bool) -> Result<(), Error> {
    if is_parent {
        write!(f, "{} {}{}§", t.ty, t.name, t.args.join(" "))
    } else {
        write!(f, "{} {}{}", t.ty, t.name, t.args.join(" "))
    }
}

fn sub_call(f: &mut Formatter, t: &TypeStruct, is_parent: bool) -> Result<(), Error> {
    if t.ty == Type::Macro && is_parent == true {
        match t.parent {
            Some(ref p) => sub_call(f, p.borrow(), true),
            _ => Ok(()),
        }
    } else {
        match t.parent {
            Some(ref p) => {
                try!(sub_call(f, p.borrow(), true));
                show(f, t, is_parent)
            },
            _ => show(f, t, is_parent),
        }
    }
}

impl Display for TypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        sub_call(f, self, false)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    Struct,
    Mod,
    Enum,
    Fn,
    Const,
    Static,
    Type,
    Variant,
    Impl,
    Use,
    Macro,
    Trait,
    Unknown,
}

impl Type {
    pub fn from(s: &str) -> Type {
        match s {
            "struct" => Type::Struct,
            "mod" => Type::Mod,
            "enum" => Type::Enum,
            "fn" => Type::Fn,
            "const" => Type::Const,
            "static" => Type::Static,
            "type" => Type::Type,
            "impl" => Type::Impl,
            "use" => Type::Use,
            "trait" => Type::Trait,
            "macro" | "macro_rules" | "macro_rules!" => Type::Macro,
            _ => Type::Variant,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Type::Struct => write!(f, "struct"),
            Type::Mod => write!(f, "mod"),
            Type::Enum => write!(f, "enum"),
            Type::Fn => write!(f, "fn"),
            Type::Const => write!(f, "const"),
            Type::Static => write!(f, "static"),
            Type::Type => write!(f, "type"),
            Type::Variant => write!(f, "variant"),
            Type::Impl => write!(f, "impl"),
            Type::Use => write!(f, "use"),
            Type::Trait => write!(f, "trait"),
            Type::Macro => write!(f, "macro"),
            _ => write!(f, "?"),
        }
    }
}
