// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(associated_consts)]

use std::marker::PhantomData;

pub struct Foo<'a> {
    f: PhantomData<&'a u32>,
}

pub struct ContentType {
    pub ttype: Foo<'static>,
    pub subtype: Foo<'static>,
    pub params: Option<Foo<'static>>,
}

impl ContentType {
    // @has const_doc/struct.ContentType.html
    // @has  - '//*[@class="docblock"]' 'Any: ContentType = ContentType{ttype: Foo{f: '
    pub const Any: ContentType = ContentType { ttype: Foo { f: PhantomData, },
                                               subtype: Foo { f: PhantomData, },
                                               params: None, };
}
