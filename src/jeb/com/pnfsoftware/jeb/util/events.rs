// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

package_name!("events");

jclass!(JebEvent, JebEvent_);

pub trait IEventSource: Instance {}
pub trait IEvent<'a>: Instance {
    fn getData(&self) -> Result<jni::objects::JObject>;
    fn getSource(&self) -> Result<Box<dyn IEventSource + '_>>;
    fn getTimestamp(&self) -> Result<i64>;
    fn getType(&self) -> Result<jni::objects::JObject>;
    fn shouldStopPropagation(&self) -> Result<bool>;
}

impl<'a, T: Instance + 'a> IEvent<'a> for T {
    fn getData(&self) -> Result<jni::objects::JObject> {
        if let Ok(obj) = call!(self, "getData", "()Ljava/lang/Object;", &[])?.l() {
            Ok(obj)
        } else {
            Err("Data is not an object".into())
        }
    }

    fn getSource(&self) -> Result<Box<dyn IEventSource>> {
        todo!()
    }

    fn getTimestamp(&self) -> Result<i64> {
        call!([i64]self, "getTimestamp", "()J", &[])
    }

    fn getType(&self) -> Result<jni::objects::JObject> {
        if let Ok(obj) = call!(self, "getType", "()Ljava/lang/Object;", &[])?.l() {
            Ok(obj)
        } else {
            Err("Type is not an object".into())
        }
    }

    fn shouldStopPropagation(&self) -> Result<bool> {
        todo!()
    }
}