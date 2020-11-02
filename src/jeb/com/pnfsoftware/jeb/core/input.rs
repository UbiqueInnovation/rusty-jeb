// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

package_name!("input");

jclass! {FileInput, FileInput_}

impl<'a> FileInput<'a> {
    constructor! {
        (from_path)
        [FileInput,FileInput_,
        "java.lang.String"]
        (path : &str) => FileInput {
            vec![jstring!(path)]
        }
    }
    constructor! {
        (from_file)
        [FileInput,FileInput_,
        "java.io.File"]
        (path : Option<file::File>) => FileInput<'a> {
            jargs!{path}
        }
    }
}