// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

package_name!("properties");

pub trait IConfiguration<'a>: crate::jeb::Instance {}
pub mod impl_ {
   package_name!("impl");

    use crate::jeb::{
        org::apache::commons::configuration2::BaseConfiguration, *,
    };

    use super::IConfiguration;

    jclass! {CommonsConfigurationWrapper,CommonsConfigurationWrapper_}

    impl<'a> CommonsConfigurationWrapper<'a> {
        constructor! {
            Box[CommonsConfigurationWrapper,CommonsConfigurationWrapper_
            ,"org/apache/commons/configuration2/Configuration"]
            (cfg : BaseConfiguration<'a>) => Box<dyn IConfiguration + 'a> {

                vec![cfg.get_obj().unwrap().into()]
            }
        }
    }
    impl<'a> IConfiguration<'a> for CommonsConfigurationWrapper<'a> {}
}