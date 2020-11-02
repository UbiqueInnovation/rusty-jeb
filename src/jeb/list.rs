// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

static PACKAGE_NAME: &str = "java/util";
use crate::jeb::*;

struct List<'a> {
    array: jni::objects::JObject<'a>,
}
impl<'a> List<'a> {
    pub fn from(jlist: &'a jni::objects::JObject<'a>) -> Result<List<'a>> {
        let env = get_vm!();
        if let jni::objects::JValue::Object(array) =
            env.call_method(*jlist, "toArray", "()[Ljava/lang/Object;", &[])?
        {
            Ok(List { array })
        } else {
            Err("no object returned".into())
        }
    }

    pub fn to_vec<T: Instance>(&self) -> Vec<T> {
        vec![]
    }
}