// Copyright (c) 2019, Facebook, Inc.
// All rights reserved.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use crate::lexable_token::LexableToken;
use crate::syntax_kind::SyntaxKind;
use crate::token_kind::TokenKind;

use std::marker::Sized;

pub use crate::syntax_generated::*;
pub use crate::syntax_type::*;

pub trait SyntaxValueType<T>
where
    Self: Sized,
{
    fn from_syntax(syntax: &SyntaxVariant<T, Self>) -> Self;
    fn from_values(ndoes: &[&Self]) -> Self;
    fn from_children(kind: SyntaxKind, offset: usize, nodes: &[&Self]) -> Self;
    fn from_token(token: &T) -> Self;

    /// Returns a range [inclusive, exclusive] for the corresponding text if meaningful
    /// (note: each implementor will either always return Some(range) or always return None).
    fn text_range(&self) -> Option<(usize, usize)>; // corresponds to extract_text in OCaml impl.
}

pub trait SyntaxValueWithKind {
    fn is_missing(&self) -> bool;
    fn token_kind(&self) -> Option<TokenKind>;
}

#[derive(Debug, Clone)]
pub struct Syntax<T, V> {
    pub syntax: SyntaxVariant<T, V>,
    pub value: V,
}

pub trait SyntaxTypeBase<'a, C> {
    type Token: LexableToken<'a>;
    type Value: SyntaxValueType<Self::Token>;

    fn make_missing(ctx: &C, offset: usize) -> Self;
    fn make_token(ctx: &C, arg: Self::Token) -> Self;
    fn make_list(ctx: &C, arg: Vec<Self>, offset: usize) -> Self
    where
        Self: Sized;

    fn value(&self) -> &Self::Value;
}

impl<'a, T, V, C> SyntaxTypeBase<'a, C> for Syntax<T, V>
where
    T: LexableToken<'a>,
    V: SyntaxValueType<T>,
{
    type Token = T;
    type Value = V;

    fn make_missing(_: &C, offset: usize) -> Self {
        let value = V::from_children(SyntaxKind::Missing, offset, &[]);
        let syntax = SyntaxVariant::Missing;
        Self::make(syntax, value)
    }

    fn make_token(_: &C, arg: T) -> Self {
        let value = V::from_token(&arg);
        let syntax = SyntaxVariant::Token(Box::new(arg));
        Self::make(syntax, value)
    }

    fn make_list(ctx: &C, arg: Vec<Self>, offset: usize) -> Self {
        // An empty list is represented by Missing; everything else is a
        // SyntaxList, even if the list has only one item.
        if arg.is_empty() {
            Self::make_missing(ctx, offset)
        } else {
            // todo: pass iter directly
            let nodes = &arg.iter().map(|x| &x.value).collect::<Vec<_>>();
            let value = V::from_children(SyntaxKind::SyntaxList, offset, nodes);
            let syntax = SyntaxVariant::SyntaxList(arg);
            Self::make(syntax, value)
        }
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

impl<'src, T, V> Syntax<T, V>
where
    T: LexableToken<'src>,
    V: SyntaxValueType<T>,
{
    pub fn make(syntax: SyntaxVariant<T, V>, value: V) -> Self {
        Self { syntax, value }
    }

    fn is_specific_token(&self, kind: TokenKind) -> bool {
        match &self.syntax {
            SyntaxVariant::Token(t) => t.kind() == kind,
            _ => false,
        }
    }

    pub fn is_public(&self) -> bool {
        self.is_specific_token(TokenKind::Public)
    }

    pub fn is_private(&self) -> bool {
        self.is_specific_token(TokenKind::Private)
    }

    pub fn is_protected(&self) -> bool {
        self.is_specific_token(TokenKind::Protected)
    }

    pub fn is_abstract(&self) -> bool {
        self.is_specific_token(TokenKind::Abstract)
    }

    pub fn is_static(&self) -> bool {
        self.is_specific_token(TokenKind::Static)
    }

    pub fn is_ampersand(&self) -> bool {
        self.is_specific_token(TokenKind::Ampersand)
    }

    pub fn is_ellipsis(&self) -> bool {
        self.is_specific_token(TokenKind::DotDotDot)
    }

    pub fn is_final(&self) -> bool {
        self.is_specific_token(TokenKind::Final)
    }

    pub fn is_async(&self) -> bool {
        self.is_specific_token(TokenKind::Async)
    }

    pub fn is_construct(&self) -> bool {
        self.is_specific_token(TokenKind::Construct)
    }

    pub fn is_void(&self) -> bool {
        self.is_specific_token(TokenKind::Void)
    }

    pub fn is_as_expression(&self) -> bool {
        self.kind() == SyntaxKind::AsExpression
    }

    pub fn is_missing(&self) -> bool {
        self.kind() == SyntaxKind::Missing
    }

    pub fn is_namespace_empty_body(&self) -> bool {
        self.kind() == SyntaxKind::NamespaceEmptyBody
    }

    pub fn syntax_node_to_list<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = &'a Self> + 'a> {
        use std::iter::{empty, once};
        match &self.syntax {
            SyntaxVariant::SyntaxList(x) => Box::new(x.iter()),
            SyntaxVariant::Missing => Box::new(empty()),
            _ => Box::new(once(self)),
        }
    }

    pub fn is_namespace_prefix(&self) -> bool {
        if let SyntaxVariant::QualifiedName(x) = &self.syntax {
            x.qualified_name_parts
                .syntax_node_to_list()
                .last()
                .map_or(false, |p| match &p.syntax {
                    SyntaxVariant::ListItem(x) => !&x.list_separator.is_missing(),
                    _ => false,
                })
        } else {
            false
        }
    }

    pub fn drain_children(&mut self) -> Vec<Self> {
        let f = |node: Self, mut acc: Vec<Self>| {
            acc.push(node);
            acc
        };
        let syntax = std::mem::replace(&mut self.syntax, SyntaxVariant::Missing);
        Self::fold_over_children_owned(&f, vec![], syntax)
    }

    pub fn replace_children(&mut self, kind: SyntaxKind, children: Vec<Self>) {
        std::mem::replace(&mut self.syntax, Syntax::from_children(kind, children));
    }

    fn get_token(&self) -> Option<&T> {
        match &self.syntax {
            SyntaxVariant::Token(t) => Some(&t),
            _ => None,
        }
    }

    pub fn leading_token(&self) -> Option<&T> {
        match self.get_token() {
            Some(token) => Some(token),
            None => {
                for node in self.iter_children() {
                    if let Some(token) = node.leading_token() {
                        return Some(token);
                    }
                }
                None
            }
        }
    }

    pub fn trailing_token(&self) -> Option<&T> {
        match self.get_token() {
            Some(token) => Some(token),
            None => {
                for node in self.iter_children().rev() {
                    if let Some(token) = node.trailing_token() {
                        return Some(token);
                    }
                }
                None
            }
        }
    }

    pub fn iter_children<'a>(&'a self) -> SyntaxChildrenIterator<'a, T, V> {
        self.syntax.iter_children()
    }
}

pub struct SyntaxChildrenIterator<'a, T, V> {
    pub syntax: &'a SyntaxVariant<T, V>,
    pub index: usize,
    pub index_back: usize,
}

impl<'src, T, V> SyntaxVariant<T, V> {
    pub fn iter_children<'a>(&'a self) -> SyntaxChildrenIterator<'a, T, V> {
        SyntaxChildrenIterator {
            syntax: &self,
            index: 0,
            index_back: 0,
        }
    }
}

impl<'a, T, V> Iterator for SyntaxChildrenIterator<'a, T, V> {
    type Item = &'a Syntax<T, V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl(true)
    }
}

impl<'a, T, V> DoubleEndedIterator for SyntaxChildrenIterator<'a, T, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_impl(false)
    }
}