// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

package_name!("dao");
pub trait IDataProvider<'a>: Instance {}

pub trait IFileDatabase<'a>: Instance {}
pub trait IFileStore<'a>: Instance {}

pub trait IUserDatabase<'a>: Instance {}

pub mod impl_ {
    package_name!("impl");
    use super::{IDataProvider, IFileDatabase, IFileStore};
    use crate::jeb::com::pnfsoftware::jeb::core::properties::IConfiguration;
    use crate::jeb::*;

    jclass! {SimpleFSFileStore, SimpleFSFileStore_}

    jclass! {JDB2Manager, JDB2Manager_}

    jclass! {DataProvider, DataProvider_}

    jclass! {UserDatabase, UserDatabase_}

    jclass! {AppDatabase, AppDatabase_}

    impl<'a> JDB2Manager<'a> {
        constructor! {
            Box[JDB2Manager,
                JDB2Manager_,
                "java/lang/String"]
                (base_dir : &str) => Box<dyn IFileDatabase<'a> + 'a> {
                    vec![jstring!(base_dir)]
                }
        }
    }

    impl<'a> SimpleFSFileStore<'a> {
        constructor! {
            Box[SimpleFSFileStore,
                SimpleFSFileStore_,
                "java/lang/String"]
                (base_dir : &str) => Box<dyn IFileStore<'a> + 'a> {
                    vec![jstring!(base_dir)]
                }
        }
    }

    impl<'a> DataProvider<'a> {
        constructor! {
            Box[DataProvider,DataProvider_,
            "com.pnfsoftware.jeb.core.dao.IUserDatabase",
            "com.pnfsoftware.jeb.core.dao.IFileDatabase",
            "com.pnfsoftware.jeb.core.dao.IFileStore",
            "com.pnfsoftware.jeb.core.dao.IFileStore",
            "com.pnfsoftware.jeb.core.dao.IApplicationDatabase",
            "com.pnfsoftware.jeb.core.properties.IConfiguration"
            ]
            (userdb :  Option<&UserDatabase>, projectdb : Option<&dyn IFileDatabase>, file_store : Option<&dyn IFileStore>, plugin_store : Option<&dyn IFileStore>, appdb : Option<&AppDatabase>, config : Option<&dyn IConfiguration>) => Box<dyn IDataProvider<'a> + 'a> {
                jargs!(userdb,projectdb, file_store, plugin_store, appdb, config)
            }

        }
    }

    impl<'a> IFileStore<'a> for SimpleFSFileStore<'a> {}
    impl<'a> IFileDatabase<'a> for JDB2Manager<'a> {}
    impl<'a> IDataProvider<'a> for DataProvider<'a> {}
}