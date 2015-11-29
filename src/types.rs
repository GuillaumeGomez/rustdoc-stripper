// Copyright 2015 Gomez Guillaume
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

use std::cmp::PartialEq;
use std::fmt::{Display, Formatter, Error};
use std::ops::Deref;

use std::fmt::Debug;

pub static MOD_COMMENT : &'static str = "=| ";
pub static FILE_COMMENT : &'static str = "=/ ";
pub static FILE : &'static str = "=! ";

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
            &EventType::Type(ref t) => writeln!(fmt, "{}", t),
            _ => Ok(())
        }
    }
}

pub struct TypeStruct {
    pub ty: Type,
    pub parent: Option<Box<TypeStruct>>,
    pub name: String,
    pub args: Vec<String>,
}

impl Debug for TypeStruct {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self.parent {
            Some(ref p) => write!(fmt, "{}|{}:{}:[{:?}]", p, self.name, self.ty, self.args),
            None => write!(fmt, "{}:{}:[{:?}]", self.name, self.ty, self.args),
        }
    }
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
}

impl PartialEq for TypeStruct {
    fn eq(&self, other: &TypeStruct) -> bool {
        self.ty == other.ty &&
        self.name == other.name &&
        self.args == other.args &&
        self.parent == other.parent
    }

    fn ne(&self, other: &TypeStruct) -> bool {
        !self.eq(other)
    }
}

impl Clone for TypeStruct {
    fn clone(&self) -> TypeStruct {
        TypeStruct {
            ty: self.ty,
            name: self.name.clone(),
            args: self.args.clone(),
            parent: match self.parent {
                Some(ref p) => Some(Box::new(p.deref().clone())),
                None => None,
            }
        }
    }

    fn clone_from(&mut self, source: &TypeStruct) {
        self.ty = source.ty;
        self.name = source.name.clone();
        self.args = source.args.clone();
        self.parent = match source.parent {
            Some(ref p) => Some(Box::new(p.deref().clone())),
            None => None,
        };
    }
}

impl Display for TypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let parent = &self.parent;
        match parent {
            &Some(ref p) => write!(f, "{}§{} {}{}", p, self.ty, self.name, self.args.join(" ")),
            &None => write!(f, "{} {}{}", self.ty, self.name, self.args.join(" ")),
        }
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