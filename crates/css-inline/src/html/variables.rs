//! Custom property (`--*`) resolution: `var()` substitution following CSS Variables Level 1.
use super::serializer::{write_declaration, write_value_bytes};
use crate::{parser, InlineError};
use hashbrown::HashTable;
use memchr::{memchr, memchr3, memchr_iter};
use rustc_hash::FxHasher;
use std::{
    hash::{Hash, Hasher},
    io::Write,
    ops::Range,
};

/// Hooks the serializer calls while writing `style` attributes. The no-op implementation makes
/// every call vanish when the option is off.
pub(crate) trait VariableScope {
    const ENABLED: bool;
    /// An element starts; its definitions are visible to its descendants.
    fn enter(&mut self);
    /// An element ends; its definitions go out of scope.
    fn exit(&mut self);
    /// Record `name: value` as a definition of the current element when `name` is `--*`.
    fn define(&mut self, name: &[u8], value: &[u8]);
    /// Record a `name<sep>value` declaration of the current element when it is `--*`.
    fn define_merged(&mut self, declaration: &[u8], separator_length: usize);
    /// All definitions of the current element are recorded: compute their values.
    fn finish_definitions(&mut self);
    /// Write `prefix`, then `name<sep>value` with its `var()` references substituted.
    /// Returns `false` without writing anything when the declaration is dropped.
    fn write_declaration<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        name: &str,
        value: &str,
        minify_css: bool,
    ) -> Result<bool, InlineError>;
    /// Same as [`VariableScope::write_declaration`] for a `name<sep>value` declaration.
    fn write_merged<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        declaration: &[u8],
        separator_length: usize,
    ) -> Result<bool, InlineError>;
}

pub(crate) struct NoVariables;

impl VariableScope for NoVariables {
    const ENABLED: bool = false;
    #[inline]
    fn enter(&mut self) {}
    #[inline]
    fn exit(&mut self) {}
    #[inline]
    fn define(&mut self, _: &[u8], _: &[u8]) {}
    #[inline]
    fn define_merged(&mut self, _: &[u8], _: usize) {}
    #[inline]
    fn finish_definitions(&mut self) {}
    #[inline]
    fn write_declaration<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        name: &str,
        value: &str,
        minify_css: bool,
    ) -> Result<bool, InlineError> {
        if !prefix.is_empty() {
            writer.write_all(prefix)?;
        }
        write_declaration(writer, name, value, minify_css)?;
        Ok(true)
    }
    #[inline]
    fn write_merged<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        declaration: &[u8],
        _: usize,
    ) -> Result<bool, InlineError> {
        if !prefix.is_empty() {
            writer.write_all(prefix)?;
        }
        writer.write_all(declaration)?;
        Ok(true)
    }
}

/// Reference chains deeper than this are treated as invalid.
const MAX_DEPTH: usize = 128;
/// Substitution results longer than this are invalid (`var()` references can double the size
/// at every level).
const MAX_VALUE_LENGTH: usize = 1 << 21;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// The value may still contain `var()`.
    Raw,
    /// On the resolution stack.
    Resolving,
    Resolved,
    /// Invalid at computed-value time; shadows an inherited definition.
    Invalid,
}

/// A custom property defined on an open element; `name` and `value` live in the arena.
struct Entry {
    name: Range<usize>,
    hash: u64,
    value: Range<usize>,
    state: State,
    /// The entry this one replaced in the index, restored when this one goes out of scope.
    shadowed: Option<usize>,
    in_cycle: bool,
    important: bool,
}

/// `var()` function found in a value, as ranges of the arena.
struct VarRef {
    start: usize,
    end: usize,
    /// `None` when the function is malformed.
    name: Option<Range<usize>>,
    /// The name contains escapes; definitions store names unescaped.
    escaped: bool,
    fallback: Option<Range<usize>>,
}

/// Custom properties of the open elements, innermost last. Resolution works on a byte arena:
/// values to substitute are appended to it and truncated once written, so steady state allocates
/// nothing.
#[derive(Default)]
pub(crate) struct CustomProperties {
    entries: Vec<Entry>,
    data: Vec<u8>,
    /// The innermost entry of each defined name, found by name hash and compared by name.
    index: HashTable<usize>,
    /// `(entries.len(), data.len())` when each open element started.
    frames: Vec<(usize, usize)>,
    /// Entries being resolved, outermost first.
    stack: Vec<usize>,
    /// Unescaped form of the name being looked up.
    scratch: Vec<u8>,
}

/// Bytes that may appear in a custom property name after `--`: ASCII alphanumerics, `_`, `-`
/// and anything non-ASCII.
static NAME_BYTES: [bool; 256] = {
    let mut table = [false; 256];
    let mut byte = 0usize;
    while byte < 256 {
        #[allow(clippy::cast_possible_truncation)]
        let b = byte as u8;
        table[byte] = b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b >= 0x80;
        byte += 1;
    }
    table
};

#[inline]
fn is_name_byte(byte: u8) -> bool {
    NAME_BYTES[usize::from(byte)]
}

#[inline]
fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0C')
}

#[inline]
fn hash_name(name: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    name.hash(&mut hasher);
    hasher.finish()
}

/// Whether the `(` at `idx` of `bytes[start..]` opens a `var()` function.
#[inline]
#[allow(clippy::arithmetic_side_effects)]
fn is_var_function(bytes: &[u8], start: usize, idx: usize) -> bool {
    idx >= start + 3
        && bytes[idx - 3..idx].eq_ignore_ascii_case(b"var")
        && (idx == start + 3 || !is_name_byte(bytes[idx - 4]))
}

/// Position of the `(` of the first `var(` in `value` (ASCII case-insensitive). Strings and
/// comments count too.
#[inline]
pub(crate) fn find_var_function(value: &[u8]) -> Option<usize> {
    memchr_iter(b'(', value).find(|&idx| is_var_function(value, 0, idx))
}

/// Whether `var(` occurs in `value` (ASCII case-insensitive). Strings and comments count too.
#[inline]
pub(crate) fn contains_var(value: &[u8]) -> bool {
    find_var_function(value).is_some()
}

/// Whether `value` has no strings, comments or escapes, so every `(` is a function start.
#[inline]
fn is_plain(value: &[u8]) -> bool {
    memchr3(b'"', b'\'', b'\\', value).is_none() && memchr(b'/', value).is_none()
}

/// Split off the `!important` flag; it is not part of a custom property's value.
fn strip_important(value: &[u8]) -> (&[u8], bool) {
    let value = value.trim_ascii();
    if memchr(b'!', value).is_none() {
        return (value, false);
    }
    // The value comes from a `&str` or from a buffer of written declarations, so it is UTF-8
    std::str::from_utf8(value)
        .ok()
        .and_then(parser::strip_important)
        .map_or((value, false), |value| (value.as_bytes(), true))
}

#[allow(clippy::arithmetic_side_effects)]
impl CustomProperties {
    fn push(&mut self, name: &[u8], value: &[u8]) {
        let (value, important) = strip_important(value);
        let hash = hash_name(name);
        let frame_start = self.frames.last().map_or(0, |frame| frame.0);
        let Self {
            entries,
            data,
            index,
            ..
        } = self;
        let slot = index.find_mut(hash, |&idx| data[entries[idx].name.clone()] == *name);
        let shadowed = match slot {
            // Within one element the later declaration wins, unless the earlier one is
            // `!important`.
            Some(&mut idx) if idx >= frame_start => {
                if entries[idx].important && !important {
                    return;
                }
                let value_start = data.len();
                data.extend_from_slice(value);
                let entry = &mut entries[idx];
                entry.value = value_start..data.len();
                entry.important = important;
                return;
            }
            Some(slot) => Some(std::mem::replace(slot, entries.len())),
            None => {
                index.insert_unique(hash, entries.len(), |&idx| entries[idx].hash);
                None
            }
        };
        let name_start = data.len();
        data.extend_from_slice(name);
        let value_start = data.len();
        data.extend_from_slice(value);
        entries.push(Entry {
            name: name_start..value_start,
            hash,
            value: value_start..data.len(),
            state: State::Raw,
            shadowed,
            in_cycle: false,
            important,
        });
    }

    /// The innermost definition of the name a `var()` refers to.
    fn lookup(&mut self, var: &VarRef) -> Option<usize> {
        let Self {
            entries,
            data,
            index,
            scratch,
            ..
        } = self;
        let mut name: &[u8] = &data[var.name.clone()?];
        if var.escaped {
            scratch.clear();
            unescape(name, scratch);
            name = scratch;
        }
        index
            .find(hash_name(name), |&idx| {
                data[entries[idx].name.clone()] == *name
            })
            .copied()
    }

    fn resolve_entry(&mut self, idx: usize) {
        let value = self.entries[idx].value.clone();
        let Some(first) = find_var_function(&self.data[value.clone()]) else {
            self.entries[idx].state = State::Resolved;
            return;
        };
        // The hint is only trusted for plain values: `find_var_function` ignores strings.
        let plain = is_plain(&self.data[value.clone()]);
        let first = plain.then_some(value.start + first);
        if self.stack.len() >= MAX_DEPTH {
            self.entries[idx].state = State::Invalid;
            return;
        }
        self.entries[idx].state = State::Resolving;
        self.stack.push(idx);
        // Dependencies first, so the output below is contiguous.
        self.resolve_references(value.clone(), idx, 0, plain, first);
        let start = self.data.len();
        let valid = !self.entries[idx].in_cycle && self.substitute(value, 0, start, plain, first);
        self.stack.pop();
        let entry = &mut self.entries[idx];
        if valid && !entry.in_cycle {
            entry.value = start..self.data.len();
            entry.state = State::Resolved;
        } else {
            self.data.truncate(start);
            entry.state = State::Invalid;
        }
    }

    /// Resolve the raw definitions the value `src` of `current` refers to and detect cycles.
    /// A fallback takes part only when it is used, i.e. the referenced property has no value,
    /// and a property in a cycle is invalid as it is, so the rest of its value is skipped.
    /// `depth` is the fallback nesting; past `MAX_DEPTH` the substitution fails anyway.
    fn resolve_references(
        &mut self,
        src: Range<usize>,
        current: usize,
        depth: usize,
        plain: bool,
        mut hint: Option<usize>,
    ) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }
        let mut pos = src.start;
        while let Some(var) = self.find_var(pos, &src, plain, hint.take()) {
            let available = match self.lookup(&var) {
                Some(found) => match self.entries[found].state {
                    State::Raw => {
                        self.resolve_entry(found);
                        self.entries[found].state == State::Resolved
                    }
                    State::Resolving => {
                        // Every property on the stack from `found` on is in the cycle.
                        let from = self.stack.iter().position(|&idx| idx == found);
                        for &idx in &self.stack[from.unwrap_or(0)..] {
                            self.entries[idx].in_cycle = true;
                        }
                        false
                    }
                    State::Resolved => true,
                    State::Invalid => false,
                },
                None => false,
            };
            if self.entries[current].in_cycle {
                return false;
            }
            if !available {
                if let Some(fallback) = var.fallback {
                    if !self.resolve_references(fallback, current, depth + 1, plain, None) {
                        return false;
                    }
                }
            }
            pos = var.end;
        }
        true
    }

    /// Append `src` with its `var()` references substituted; `false` when the result is invalid.
    /// `start` is where the output of the outermost call begins and `hint` the position of the
    /// first `var(` when known.
    fn substitute(
        &mut self,
        src: Range<usize>,
        depth: usize,
        start: usize,
        plain: bool,
        mut hint: Option<usize>,
    ) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }
        let mut pos = src.start;
        while let Some(var) = self.find_var(pos, &src, plain, hint.take()) {
            self.data.extend_from_within(pos..var.start);
            if var.name.is_none() {
                return false;
            }
            let value = self
                .lookup(&var)
                .filter(|&idx| self.entries[idx].state == State::Resolved)
                .map(|idx| self.entries[idx].value.clone());
            if let Some(value) = value {
                self.data.extend_from_within(value);
            } else if let Some(fallback) = var.fallback {
                if !self.substitute(fallback, depth + 1, start, plain, None) {
                    return false;
                }
            } else {
                return false;
            }
            if self.data.len() - start > MAX_VALUE_LENGTH {
                return false;
            }
            pos = var.end;
        }
        self.data.extend_from_within(pos..src.end);
        true
    }

    /// Find the next `var()` in `src` at or after `pos`, skipping strings and comments unless
    /// `src` is plain. `hint` is the position of its `(` when already known.
    fn find_var(
        &self,
        mut pos: usize,
        src: &Range<usize>,
        plain: bool,
        hint: Option<usize>,
    ) -> Option<VarRef> {
        let data = &self.data;
        let end = src.end;
        if let Some(idx) = hint {
            return Some(self.parse_var(idx - 3, idx + 1, end));
        }
        if plain {
            return memchr_iter(b'(', &data[pos..end])
                .map(|idx| pos + idx)
                .find(|&idx| is_var_function(data, src.start, idx))
                .map(|idx| self.parse_var(idx - 3, idx + 1, end));
        }
        while pos < end {
            match data[pos] {
                b'"' | b'\'' => pos = skip_string(data, pos, end),
                b'/' if pos + 1 < end && data[pos + 1] == b'*' => {
                    pos = skip_comment(data, pos, end);
                }
                b'\\' => pos += 2,
                b'(' if is_var_function(data, src.start, pos) => {
                    return Some(self.parse_var(pos - 3, pos + 1, end));
                }
                _ => pos += 1,
            }
        }
        None
    }

    /// Parse `var(` whose arguments start at `pos`.
    fn parse_var(&self, start: usize, pos: usize, end: usize) -> VarRef {
        let data = &self.data;
        let mut pos = skip_whitespace_and_comments(data, pos, end);
        let name_start = pos;
        let mut name = None;
        let mut escaped = false;
        if pos + 2 <= end && &data[pos..pos + 2] == b"--" {
            pos += 2;
            while pos < end {
                match data[pos] {
                    b'\\' => {
                        escaped = true;
                        pos = skip_escape(data, pos, end);
                    }
                    byte if is_name_byte(byte) => pos += 1,
                    _ => break,
                }
            }
            // `--` alone is reserved and not a custom property name.
            if pos > name_start + 2 {
                name = Some(name_start..pos);
            }
        }
        pos = skip_whitespace_and_comments(data, pos, end);
        match data.get(pos).filter(|_| pos < end) {
            None | Some(b')') => VarRef {
                start,
                end: (pos + 1).min(end),
                name,
                escaped,
                fallback: None,
            },
            Some(b',') => {
                let close = find_closing_paren(data, pos + 1, end);
                VarRef {
                    start,
                    end: (close + 1).min(end),
                    name,
                    escaped,
                    fallback: Some(trim(data, pos + 1, close)),
                }
            }
            Some(_) => {
                let close = find_closing_paren(data, pos, end);
                VarRef {
                    start,
                    end: (close + 1).min(end),
                    name: None,
                    escaped: false,
                    fallback: None,
                }
            }
        }
    }

    /// Write `head` followed by `value` with its `var()` references substituted; `first` is the
    /// position of its first `var(`. Nothing is written when the result is invalid: the
    /// declaration is dropped, where the specification computes it as `unset`, which email
    /// clients do not support either.
    fn write_substituted<W: Write>(
        &mut self,
        writer: &mut W,
        head: &[&[u8]],
        value: &[u8],
        first: usize,
    ) -> Result<bool, InlineError> {
        let trimmed = value.trim_ascii();
        let first = first - (value.len() - value.trim_ascii_start().len());
        let plain = is_plain(trimmed);
        let start = self.data.len();
        self.data.extend_from_slice(trimmed);
        let output = self.data.len();
        let hint = plain.then_some(start + first);
        let valid = self.substitute(start..output, 0, output, plain, hint)
            && !self.data[output..].trim_ascii().is_empty();
        if valid {
            for part in head {
                writer.write_all(part)?;
            }
            write_value_bytes(writer, &self.data[output..])?;
        }
        self.data.truncate(start);
        Ok(valid)
    }
}

#[allow(clippy::arithmetic_side_effects)]
fn skip_string(data: &[u8], pos: usize, end: usize) -> usize {
    let quote = data[pos];
    let mut pos = pos + 1;
    while pos < end {
        match data[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return pos + 1,
            // Unescaped newlines end strings.
            b'\n' => return pos,
            _ => pos += 1,
        }
    }
    end
}

/// Position after the escape starting with the `\` at `pos`: up to six hex digits and one
/// whitespace, or a single code point. The HTML parser turns CR LF into LF, so one byte of
/// whitespace is enough.
#[allow(clippy::arithmetic_side_effects)]
fn skip_escape(data: &[u8], pos: usize, end: usize) -> usize {
    let digits = pos + 1;
    let mut pos = digits;
    while pos < end && pos - digits < 6 && data[pos].is_ascii_hexdigit() {
        pos += 1;
    }
    if pos == digits {
        return (pos + 1).min(end);
    }
    if pos < end && is_whitespace(data[pos]) {
        pos += 1;
    }
    pos
}

/// Append the identifier `name` to `out` with its escapes decoded, the way declaration names
/// are parsed. `name` is a slice of a `str` ending at an ASCII byte and starts with `--`.
fn unescape(name: &[u8], out: &mut Vec<u8>) {
    let name = std::str::from_utf8(name).expect("name is a slice of a str");
    let mut parser = cssparser::Parser::new(name);
    let ident = parser.expect_ident().expect("name starts with `--`");
    out.extend_from_slice(ident.as_bytes());
}

#[allow(clippy::arithmetic_side_effects)]
fn skip_comment(data: &[u8], pos: usize, end: usize) -> usize {
    let after = pos + 2;
    memchr::memmem::find(&data[after..end], b"*/").map_or(end, |idx| after + idx + 2)
}

#[allow(clippy::arithmetic_side_effects)]
fn skip_whitespace_and_comments(data: &[u8], mut pos: usize, end: usize) -> usize {
    while pos < end {
        match data[pos] {
            byte if is_whitespace(byte) => pos += 1,
            b'/' if pos + 1 < end && data[pos + 1] == b'*' => pos = skip_comment(data, pos, end),
            _ => break,
        }
    }
    pos
}

/// Position of the `)` closing the block whose contents start at `pos`; `end` when unclosed.
#[allow(clippy::arithmetic_side_effects)]
fn find_closing_paren(data: &[u8], mut pos: usize, end: usize) -> usize {
    let mut depth = 0usize;
    while pos < end {
        match data[pos] {
            b'"' | b'\'' => pos = skip_string(data, pos, end),
            b'/' if pos + 1 < end && data[pos + 1] == b'*' => pos = skip_comment(data, pos, end),
            b'\\' => pos += 2,
            b'(' => {
                depth += 1;
                pos += 1;
            }
            b')' => {
                if depth == 0 {
                    return pos;
                }
                depth -= 1;
                pos += 1;
            }
            _ => pos += 1,
        }
    }
    end
}

#[allow(clippy::arithmetic_side_effects)]
fn trim(data: &[u8], mut start: usize, mut end: usize) -> Range<usize> {
    while start < end && is_whitespace(data[start]) {
        start += 1;
    }
    while end > start && is_whitespace(data[end - 1]) {
        end -= 1;
    }
    start..end
}

impl VariableScope for CustomProperties {
    const ENABLED: bool = true;

    fn enter(&mut self) {
        self.frames.push((self.entries.len(), self.data.len()));
    }

    fn exit(&mut self) {
        let (entries, data) = self.frames.pop().expect("exit without enter");
        for (offset, entry) in self.entries.drain(entries..).enumerate().rev() {
            let idx = entries.saturating_add(offset);
            match entry.shadowed {
                Some(previous) => {
                    *self
                        .index
                        .find_mut(entry.hash, |&i| i == idx)
                        .expect("defined name is indexed") = previous;
                }
                None => {
                    self.index
                        .find_entry(entry.hash, |&i| i == idx)
                        .expect("defined name is indexed")
                        .remove();
                }
            }
        }
        self.data.truncate(data);
    }

    fn define(&mut self, name: &[u8], value: &[u8]) {
        if name.starts_with(b"--") {
            self.push(name, value);
        }
    }

    fn define_merged(&mut self, declaration: &[u8], separator_length: usize) {
        if declaration.starts_with(b"--") {
            if let Some(colon) = memchr(b':', declaration) {
                let value_start = colon
                    .saturating_add(separator_length)
                    .min(declaration.len());
                self.push(&declaration[..colon], &declaration[value_start..]);
            }
        }
    }

    fn finish_definitions(&mut self) {
        let start = self.frames.last().map_or(0, |frame| frame.0);
        for idx in start..self.entries.len() {
            if self.entries[idx].state == State::Raw {
                self.resolve_entry(idx);
            }
        }
    }

    fn write_declaration<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        name: &str,
        value: &str,
        minify_css: bool,
    ) -> Result<bool, InlineError> {
        if name.starts_with("--") {
            return Ok(false);
        }
        let Some(first) = find_var_function(value.as_bytes()) else {
            writer.write_all(prefix)?;
            write_declaration(writer, name, value, minify_css)?;
            return Ok(true);
        };
        let separator: &[u8] = if minify_css { b":" } else { b": " };
        self.write_substituted(
            writer,
            &[prefix, name.as_bytes(), separator],
            value.as_bytes(),
            first,
        )
    }

    fn write_merged<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: &[u8],
        declaration: &[u8],
        separator_length: usize,
    ) -> Result<bool, InlineError> {
        if declaration.starts_with(b"--") {
            return Ok(false);
        }
        let value_start = memchr(b':', declaration).map_or(declaration.len(), |colon| {
            colon
                .saturating_add(separator_length)
                .min(declaration.len())
        });
        let (head, value) = declaration.split_at(value_start);
        let Some(first) = find_var_function(value) else {
            writer.write_all(prefix)?;
            writer.write_all(declaration)?;
            return Ok(true);
        };
        self.write_substituted(writer, &[prefix, head], value, first)
    }
}
