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