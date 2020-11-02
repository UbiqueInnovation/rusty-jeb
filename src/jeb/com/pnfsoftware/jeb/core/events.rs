// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::jeb::*;
package_name!("events");

use std::convert::TryInto;

jclass! {ClientNotification, ClientNotification_}

#[derive(Debug)]
pub enum ClientNotificationLevel {
    ErrorLevel,
    InfoLevel,
    WarningLevel,
}

impl<'a> ClientNotification<'a> {
    pub fn getMessage(&self) -> Result<String> {
        call!([String]self,"getMessage", "()Ljava/lang/String;", &[])
    }
    pub fn getLevel(&self) -> Result<ClientNotificationLevel> {
        let res = call!(
            self,
            "getLevel",
            "()Lcom.pnfsoftware.jeb.core.events.ClientNotificationLevel;",
            &[]
        )?;
        Ok(res.l()?.try_into()?)
    }
}

impl<'a> TryFrom<jni::objects::JObject<'a>> for ClientNotification<'a> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: jni::objects::JObject<'a>,
    ) -> core::result::Result<Self, Self::Error> {
        let env = get_vm!();
        let class = env.find_class(normalize!(
            "com.pnfsoftware.jeb.core.events.ClientNotification"
        ))?;
        if env.is_instance_of(value, class)? {
            Ok(ClientNotification(value.into()))
        } else {
            Err("Is not a ClientNotification".into())
        }
    }
}

impl<'a> TryFrom<jni::objects::JObject<'a>> for ClientNotificationLevel {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: jni::objects::JObject<'a>,
    ) -> core::result::Result<Self, Self::Error> {
        let env = get_vm!();
        let res =
            env.call_method(value, "toString", "()Ljava/lang/String;", &[])?;
        let string: String = env.get_string(res.l()?.into())?.into();
        match string.as_str() {
            "ERROR" => Ok(ClientNotificationLevel::ErrorLevel),
            "INFO" => Ok(ClientNotificationLevel::InfoLevel),
            "WARNING" => Ok(ClientNotificationLevel::WarningLevel),
            _ => Err("not a valid notificationlevel".into()),
        }
    }
}