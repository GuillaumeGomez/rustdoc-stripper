#![feature(rustc_private)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lexer;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::prelude::*;
use std::io::{self, Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use rustc_ast::ast::{AttrKind, Crate, Item, ItemKind};
use rustc_ast::visit::{walk_crate, walk_item, Visitor};
use rustc_data_structures::sync::{Lrc, Send};
use rustc_errors::emitter::{Emitter, EmitterWriter};
use rustc_errors::translation::Translate;
use rustc_errors::{ColorConfig, Diagnostic, Handler, Level as DiagnosticLevel};
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_lexer::{Cursor, TokenKind};
use rustc_parse::new_parser_from_file;
use rustc_parse::parser::Parser;
use rustc_session::parse::ParseSess;
use rustc_span::edition::Edition;
use rustc_span::source_map::{FilePathMapping, SourceMap};
use rustc_span::{BytePos, FileName};
use rustc_span::{Span, DUMMY_SP};

pub const OUTPUT_COMMENT_FILE: &str = "comments.md";

/// let's give it a try?
pub fn strip_comments<F: Write>(
    path: &str,
    out_file: &mut F,
    ignore_macros: bool,
) -> Result<(), String> {
    let path = PathBuf::from(path);

    rustc_span::create_session_if_not_set_then(Edition::Edition2024, |_| {
        let parser_session = create_parser_session();
        let mut parser = create_parser(&path, &parser_session)?;
        let krate = parse_crate(&mut parser)?;

        walk_crate(&mut DocCommentVisitor::new(&parser_session), &krate);
        Ok(())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    position: usize,
    len: usize,
}

impl Position {
    fn new(position: BytePos, len: BytePos) -> Self {
        Self {
            position: position.0 as _,
            len: len.0 as _,
        }
    }
}

struct DocCommentVisitor<'a> {
    current_path: Vec<&'static str>,
    sess: &'a ParseSess,
    previous_span: HashMap<FileName, Span>,
    /// Key is the file name and the value is a vector of `Span` (where the doc comment starts and
    /// ends) and the "path" of the item (ie `a::b::c`).
    doc_comments: HashMap<FileName, Vec<(Position, String)>>,
}

impl<'a> Drop for DocCommentVisitor<'a> {
    fn drop(&mut self) {
        eprintln!("==> {:?}", self.doc_comments);
    }
}

impl<'a> DocCommentVisitor<'a> {
    fn new(sess: &'a ParseSess) -> Self {
        Self {
            doc_comments: HashMap::with_capacity(10),
            current_path: Vec::with_capacity(10),
            previous_span: HashMap::with_capacity(10),
            sess,
        }
    }

    fn add_if_local(&mut self, span: Span, item_path: String) {
        let loc = self.sess.source_map().lookup_char_pos(span.lo());
        if loc.file.cnum != LOCAL_CRATE {
            return;
        }
        // We create the position in the file.
        let pos = Position::new(span.lo() - loc.file.start_pos, span.hi() - span.lo());
        match self.doc_comments.entry(loc.file.name.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push((pos, item_path));
            }
            Entry::Vacant(e) => {
                let mut v = Vec::with_capacity(100);
                v.push((pos, item_path));
                e.insert(v);
            }
        }
    }

    fn get_ignore_next(&self, current_ignore_next: bool, current: Span, next: Span) -> bool {
        let mut ignore_next = current_ignore_next;
        let lo = current.hi() + BytePos(1);
        let hi = next.lo() - BytePos(1);
        let in_between = match self
            .sess
            .source_map()
            .span_to_snippet(next.with_lo(lo).with_hi(hi))
        {
            Ok(s) => s,
            _ => return ignore_next,
        };
        let mut cursor = Cursor::new(&in_between);
        let mut text = in_between.as_str();
        loop {
            let token = cursor.advance_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            let (token_text, rest) = text.split_at(token.len as _);
            match token.kind {
                TokenKind::LineComment { doc_style: None }
                | TokenKind::BlockComment {
                    doc_style: None, ..
                } => {
                    if text.contains(" doc-stripper-ignore-next ")
                        || text.contains(" doc-stripper-ignore-next\n")
                        || text.ends_with(" doc-stripper-ignore-next")
                    {
                        ignore_next = true;
                    } else if text.contains(" doc-stripper-ignore-next-stop ")
                        || text.contains(" doc-stripper-ignore-next-stop\n")
                        || text.ends_with(" doc-stripper-ignore-next-stop")
                    {
                        ignore_next = false;
                    }
                }
                _ => {}
            }
            text = rest;
        }
        ignore_next
    }

    fn get_previous_span_for_file(&self, span: Span) -> Span {
        let file = self.sess.source_map().lookup_source_file(span.lo());
        if let Some(prev) = self.previous_span.get(&file.name) {
            span.with_lo(prev.hi()).with_hi(span.lo())
        } else {
            span.with_lo(file.start_pos).with_hi(span.lo())
        }
    }

    fn check_attributes(&mut self, i: &Item) {
        if i.attrs.is_empty() {
            return;
        }
        let mut path: Option<String> = None;
        let mut ignore_next =
            self.get_ignore_next(false, self.get_previous_span_for_file(i.span), i.span);
        let mut attrs = i.attrs.iter().peekable();

        while let Some(attr) = attrs.next() {
            if !ignore_next && attr.doc_str().is_some() {
                self.add_if_local(
                    attr.span,
                    path.get_or_insert_with(|| self.current_path.join("::"))
                        .clone(),
                );
            }
            if let Some(next_attr) = attrs.peek() {
                ignore_next = self.get_ignore_next(ignore_next, attr.span, next_attr.span);
            }
        }
    }
}

impl<'a> Visitor<'a> for DocCommentVisitor<'a> {
    fn visit_item(&mut self, i: &'a Item) {
        let ident: &'static str = unsafe { std::mem::transmute(i.ident.as_str()) };
        if !ident.is_empty() {
            eprintln!("entering ===> {:?}", ident);
            self.current_path.push(ident);
            self.check_attributes(i);
        }
        walk_item(self, i);
        if !ident.is_empty() {
            self.current_path.pop();
            eprintln!("leving <=== {:?}", ident);
        }
    }
}

/// Emit errors against every files.
struct SilentOnIgnoredFilesEmitter {
    source_map: Lrc<SourceMap>,
    emitter: Box<dyn Emitter + Send>,
    has_non_ignorable_parser_errors: bool,
    can_reset: Lrc<AtomicBool>,
}

impl SilentOnIgnoredFilesEmitter {
    fn handle_non_ignoreable_error(&mut self, db: &Diagnostic) {
        self.has_non_ignorable_parser_errors = true;
        self.can_reset.store(false, Ordering::Release);
        self.emitter.emit_diagnostic(db);
    }
}

impl Translate for SilentOnIgnoredFilesEmitter {
    fn fluent_bundle(&self) -> Option<&Lrc<rustc_errors::FluentBundle>> {
        self.emitter.fluent_bundle()
    }

    fn fallback_fluent_bundle(&self) -> &rustc_errors::FluentBundle {
        self.emitter.fallback_fluent_bundle()
    }
}

impl Emitter for SilentOnIgnoredFilesEmitter {
    fn source_map(&self) -> Option<&Lrc<SourceMap>> {
        None
    }
    fn emit_diagnostic(&mut self, db: &Diagnostic) {
        if db.level() == DiagnosticLevel::Fatal {
            return self.handle_non_ignoreable_error(db);
        }
        if let Some(primary_span) = &db.span.primary_span() {
            let file_name = self.source_map.span_to_filename(*primary_span);
            if let FileName::Real(rustc_span::RealFileName::LocalPath(_)) = file_name {
                if !self.has_non_ignorable_parser_errors {
                    self.can_reset.store(true, Ordering::Release);
                }
                return;
            };
        }
        self.handle_non_ignoreable_error(db);
    }
}

// Emitter which discards every error.
struct SilentEmitter;

impl Emitter for SilentEmitter {
    fn source_map(&self) -> Option<&Lrc<SourceMap>> {
        None
    }
    fn emit_diagnostic(&mut self, _db: &Diagnostic) {}
}

impl Translate for SilentEmitter {
    fn fluent_bundle(&self) -> Option<&Lrc<rustc_errors::FluentBundle>> {
        None
    }
    fn fallback_fluent_bundle(&self) -> &rustc_errors::FluentBundle {
        panic!("silent emitter attempted to translate a diagnostic");
    }
}

fn silent_emitter() -> Box<dyn Emitter + Send> {
    Box::new(SilentEmitter {})
}

fn default_handler(
    source_map: Lrc<SourceMap>,
    can_reset: Lrc<AtomicBool>,
    hide_parse_errors: bool,
) -> Handler {
    let supports_color = term::stderr().map_or(false, |term| term.supports_color());
    let color_cfg = if supports_color {
        ColorConfig::Auto
    } else {
        ColorConfig::Never
    };

    let emitter = if hide_parse_errors {
        silent_emitter()
    } else {
        let fallback_bundle =
            rustc_errors::fallback_fluent_bundle(rustc_errors::DEFAULT_LOCALE_RESOURCES, false);
        Box::new(EmitterWriter::stderr(
            color_cfg,
            Some(source_map.clone()),
            None,
            fallback_bundle,
            false,
            false,
            None,
            false,
        ))
    };
    Handler::with_emitter(
        true,
        None,
        Box::new(SilentOnIgnoredFilesEmitter {
            has_non_ignorable_parser_errors: false,
            source_map,
            emitter,
            can_reset,
        }),
    )
}

fn create_parser_session() -> ParseSess {
    let source_map = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let can_reset_errors = Lrc::new(AtomicBool::new(false));

    let handler = default_handler(
        Lrc::clone(&source_map),
        Lrc::clone(&can_reset_errors),
        false,
    );
    ParseSess::with_span_handler(handler, source_map)
}

fn create_parser<'a>(file: &Path, sess: &'a ParseSess) -> Result<Parser<'a>, String> {
    catch_unwind(AssertUnwindSafe(move || {
        new_parser_from_file(sess, file, None)
    }))
    .map_err(|e| format!("failed to create parser: {:?}", e))
}

fn parse_crate(parser: &mut Parser) -> Result<Crate, String> {
    let mut parser = AssertUnwindSafe(parser);

    match catch_unwind(move || parser.parse_crate_mod()) {
        Ok(Ok(k)) => Ok(k),
        Ok(Err(mut db)) => {
            db.emit();
            Err(String::new())
        }
        Err(e) => Err(format!("parser panicked: {:?}", e)),
    }
}
