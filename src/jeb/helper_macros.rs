use crate::jeb::*;

macro_rules! package_name {
    ($name:expr) => {
        use crate::jeb::*;
        const PACKAGE_NAME : &str = concatcp!(super::PACKAGE_NAME,"/", $name);
    };
}


macro_rules! jcall {
    ([$signature:expr][$concrete_type:expr]fn $fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        fn $fname<'t>(&self,$($arg : $typ),*) -> Result<'t,$res> {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let obj =  env.call_method(obj,stringify!($fname),$signature, &args)?;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into() {
                Ok(
                    Box::new(
                        $concrete_type(
                            obj.into()
                        )
                    )
                )
            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Vec[$signature:expr][$concrete_type:expr]fn $fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        fn $fname(&self,$($arg : $typ),*) -> Result<$res> {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into() {
                let result = env.call_method(obj,stringify!($fname),$signature, &args)?.into();
                if let jni::objects::JValue::Object(array) = result {
                    let list = jni::objects::JList::from_env(&env, array)?;
                    let mut result : $res = vec![];
                    for element in list.iter()? {
                        result.push(
                            Box::new(
                                $concrete_type(
                                    element.into()
                                )
                            )
                        )
                    }
                    Ok(result)
                } else {
                    Err("return type is not a object".into())
                }

            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Vec<$interface:tt>[$concrete_type:expr]fn $fname:ident()) => {
        fn $fname(&self) -> Result<Vec<Box<dyn $interface + '_>>> {
            let env = VM.attach_current_thread_permanently()?;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into(){
                let result = env.call_method(obj,stringify!($fname),"()Ljava/util/List;", &[])?.into();
                if let jni::objects::JValue::Object(array) = result {
                    let list = jni::objects::JList::from_env(&env, array)?;
                    let mut result : Vec<Box<dyn $interface + '_>> = vec![];
                    for element in list.iter()? {
                        result.push(
                            Box::new(
                                $concrete_type(
                                    element.into()
                                )
                            )
                        )
                    }
                    Ok(result)
                } else {
                    Err("return type is not a object".into())
                }

            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Box[$signature:expr][$concrete_type:ty]$fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        pub fn $fname($($arg : $typ),*) -> Result<()> {
            let env = VM.attach_current_thread_permanently()?
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.0 {
                Ok(env.call_method(obj,stringify!($fname),$signature, &args)?)
            } else {
                Err("Instance is not object".into())
            }
        }
    };
    ([$signature:expr][$concrete_type:ty]$fname:ident($($arg:ident : $typ:ty),*) $conversion:block) => {
        pub fn $fname($($arg : $typ),*) -> Result<()> {
            let env = VM.attach_current_thread_permanently()?
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.0 {
                Ok(env.call_method(obj,stringify!($fname),$signature, &args)?.into())

            } else {
                Err("Instance is not object".into())
            }
        }
    };
}

macro_rules! call_object {
    ($obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        env.call_method($obj, $name, normalize!($signature), $args)
    }};
    ([String]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let result: jni::objects::JValue =
            env.call_method($obj, $name, normalize!($signature), $args)?;
        let jstring: jni::objects::JString = result.l()?.into();
        let string: String = env.get_string(jstring)?.into();
        Ok(string)
    }};
    ([Bool]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.z()?)
    }};
    ([i32]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.i()?)
    }};
    ([i64]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.j()?)
    }};
}

macro_rules! call {
    ($obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
           call_object!(obj, $name, $signature, $args)
        } else {
            Err(jni::errors::Error::WrongJValueType("Expected JObject", ""))
        }
    };
    ([String]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([String]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([Bool]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
           call_object!([Bool]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([i32]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([i32]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([i64]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([i64]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
}

macro_rules! normalize {
    ($x:expr) => {
        $x.replace(".", "/").trim()
    };
}

macro_rules! jargs {
    ($($x:ident),+) => {

        vec![
        $(
            $x.as_ref().map_or(jni::objects::JObject::null().into(), |x| {
               x.get_obj().unwrap().into()
            }),
        )*
        ]
    };
}

macro_rules! get_vm {
    () => {
        VM.attach_current_thread_permanently()?
    };
}
macro_rules! get_vm_unwrap {
    () => {
        VM.attach_current_thread_permanently()
            .expect("Could not attach vm")
    };
}

macro_rules! jstring {
    ($x:ident) => {
        get_vm!().new_string($x)?.into()
    };
}

macro_rules! jclass {
    ($x:ident, $y:ident) => {
        #[derive(Instance)]
        pub struct $x<'a>(pub jni::objects::JValue<'a>);
        #[derive(ClassFromStr)]
        struct $y;
    };
    ($x:ident, $y:ident, $consume:ty) => {
        #[derive(Instance)]
        pub struct $x<'a>(pub jni::objects::JValue<'a>, pub $consume);
        #[derive(ClassFromStr)]
        struct $y;
    };
}

macro_rules! constructor {
    (Box[$instance:ident,$instance_:ident $(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty  $conversion:block ) => {
        pub fn new<'t>($($arg : $typ),*) -> Result<'t,$res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args : Vec<jni::objects::JValue> = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements{
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
            let obj = env.new_object($instance_,ctor_sig, &args)?;
            Ok(
                Box::new(
                    $instance(obj.into())
                )
            )
        }
    };
    ([$instance:ident,$instance_:ident$(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty $conversion:block) => {
        pub fn new<'t>($($arg : $typ),*) -> Result<'t, $res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements {
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
            let obj = env.new_object($instance_,ctor_sig, &args)?;
            let global_ref = env.new_global_ref(obj)?;
            Ok(
                $instance(obj.into(), global_ref)
            )
        }
    };
    (($constructor_name:ident)[$instance:ident,$instance_:ident$(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty $conversion:block) => {
        pub fn $constructor_name<'t>($($arg : $typ),*) -> Result<'t,$res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements {
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
           let obj = env.new_object($instance_,ctor_sig, &args)?;
            Ok(
                $instance(obj.into())
            )
        }
    };
}
macro_rules! propagate_interface {
    ([$base:tt][$super_interface:tt][$concrete_type:ident] fn $fname:ident ($($arg:ident : $typ:ty),*)) => {
        fn $fname(&self, $($arg : $typ),*) -> Result<Box<dyn $super_interface + '_>> {
            let res = $base::$fname(self, $($arg),*)?.into();
            box_ok!($concrete_type(res))
        }
    }
}

macro_rules! box_ok {
    ($type:ident ($arg:ident)) => {
        Ok(Box::new($type($arg)))
    };
}
#[allow(unused_macros)]
macro_rules! jcall {
    ([$signature:expr][$concrete_type:expr]fn $fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        fn $fname<'t>(&self,$($arg : $typ),*) -> Result<'t,$res> {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let obj =  env.call_method(obj,stringify!($fname),$signature, &args)?;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into() {
                Ok(
                    Box::new(
                        $concrete_type(
                            obj.into()
                        )
                    )
                )
            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Vec[$signature:expr][$concrete_type:expr]fn $fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        fn $fname(&self,$($arg : $typ),*) -> Result<$res> {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into() {
                let result = env.call_method(obj,stringify!($fname),$signature, &args)?.into();
                if let jni::objects::JValue::Object(array) = result {
                    let list = jni::objects::JList::from_env(&env, array)?;
                    let mut result : $res = vec![];
                    for element in list.iter()? {
                        result.push(
                            Box::new(
                                $concrete_type(
                                    element.into()
                                )
                            )
                        )
                    }
                    Ok(result)
                } else {
                    Err("return type is not a object".into())
                }

            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Vec<$interface:tt>[$concrete_type:expr]fn $fname:ident()) => {
        fn $fname(&self) -> Result<Vec<Box<dyn $interface + '_>>> {
            let env = VM.attach_current_thread_permanently()?;
            if let jni::objects::JValue::Object(obj) = self.get_obj()?.into(){
                let result = env.call_method(obj,stringify!($fname),"()Ljava/util/List;", &[])?.into();
                if let jni::objects::JValue::Object(array) = result {
                    let list = jni::objects::JList::from_env(&env, array)?;
                    let mut result : Vec<Box<dyn $interface + '_>> = vec![];
                    for element in list.iter()? {
                        result.push(
                            Box::new(
                                $concrete_type(
                                    element.into()
                                )
                            )
                        )
                    }
                    Ok(result)
                } else {
                    Err("return type is not a object".into())
                }

            } else {
                Err("Instance is not object".into())
            }
        }
    };
    (Box[$signature:expr][$concrete_type:ty]$fname:ident($($arg:ident : $typ:ty),*) -> $res:ty $conversion:block) => {
        pub fn $fname($($arg : $typ),*) -> Result<()> {
            let env = VM.attach_current_thread_permanently()?
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.0 {
                Ok(env.call_method(obj,stringify!($fname),$signature, &args)?)
            } else {
                Err("Instance is not object".into())
            }
        }
    };
    ([$signature:expr][$concrete_type:ty]$fname:ident($($arg:ident : $typ:ty),*) $conversion:block) => {
        pub fn $fname($($arg : $typ),*) -> Result<()> {
            let env = VM.attach_current_thread_permanently()?
            let args = $conversion;
            if let jni::objects::JValue::Object(obj) = self.0 {
                Ok(env.call_method(obj,stringify!($fname),$signature, &args)?.into())

            } else {
                Err("Instance is not object".into())
            }
        }
    };
}

macro_rules! call_object {
    ($obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        env.call_method($obj, $name, normalize!($signature), $args)
    }};
    ([String]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let result: jni::objects::JValue =
            env.call_method($obj, $name, normalize!($signature), $args)?;
        let jstring: jni::objects::JString = result.l()?.into();
        let string: String = env.get_string(jstring)?.into();
        Ok(string)
    }};
    ([Bool]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.z()?)
    }};
    ([i32]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.i()?)
    }};
    ([i64]$obj:expr, $name:expr, $signature:expr, $args:expr) => {{
        let env = get_vm!();
        let res = env.call_method($obj, $name, normalize!($signature), $args)?;
        Ok(res.j()?)
    }};
}

macro_rules! call {
    ($obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
           let res = call_object!(obj, $name, $signature, $args)?;
           let env = get_vm!();
           if env.is_same_object(res.l()?, jni::objects::JObject::null())? {
            Err("Null pointer".into())
           } else {
            Ok(res)
           }
           
        } else {
            Err("self is not an object")
        }
    };
    ([String]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([String]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([Bool]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
           call_object!([Bool]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([i32]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([i32]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
    ([i64]$obj:expr, $name:expr, $signature:expr, $args:expr) => {
        if let Ok(obj) = $obj.get_obj() {
            call_object!([i64]obj, $name, $signature, $args)
        } else {
            Err("Instance is not an object".into())
        }
    };
}

macro_rules! normalize {
    ($x:expr) => {
        $x.replace(".", "/").trim()
    };
}

macro_rules! jargs {
    ($($x:ident),+) => {

        vec![
        $(
            $x.as_ref().map_or(jni::objects::JObject::null().into(), |x| {
               x.get_obj().unwrap().into()
            }),
        )*
        ]
    };
}

macro_rules! get_vm {
    () => {
        VM.attach_current_thread_permanently()?
    };
}
macro_rules! get_vm_unwrap {
    () => {
        VM.attach_current_thread_permanently()
            .expect("Could not attach vm")
    };
}

macro_rules! jstring {
    ($x:ident) => {
        get_vm!().new_string($x)?.into()
    };
}

macro_rules! jclass {
    ($x:ident, $y:ident) => {
        #[derive(Instance)]
        pub struct $x<'a>(pub jni::objects::JValue<'a>);
        #[derive(ClassFromStr)]
        struct $y;
    };
    ($x:ident, $y:ident, $consume:ty) => {
        #[derive(Instance)]
        pub struct $x<'a>(pub jni::objects::JValue<'a>, pub $consume);
        #[derive(ClassFromStr)]
        struct $y;
    };
}

macro_rules! constructor {
    (Box[$instance:ident,$instance_:ident $(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty  $conversion:block ) => {
        pub fn new<'t>($($arg : $typ),*) -> Result<'t,$res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args : Vec<jni::objects::JValue> = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements{
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
            let obj = env.new_object($instance_,ctor_sig, &args)?;
            Ok(
                Box::new(
                    $instance(obj.into())
                )
            )
        }
    };
    ([$instance:ident,$instance_:ident$(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty $conversion:block) => {
        pub fn new<'t>($($arg : $typ),*) -> Result<'t, $res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements {
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
            let obj = env.new_object($instance_,ctor_sig, &args)?;
            let global_ref = env.new_global_ref(obj)?;
            Ok(
                $instance(obj.into(), global_ref)
            )
        }
    };
    (($constructor_name:ident)[$instance:ident,$instance_:ident$(,$signature:expr)*]($($arg:ident : $typ:ty),*) => $res:ty $conversion:block) => {
        pub fn $constructor_name<'t>($($arg : $typ),*) -> Result<'t,$res>   {
            let env = VM.attach_current_thread_permanently()?;
            let args = $conversion;
            let mut ctor_sig = String::from("(");
            let signature_elements : Vec<&str> = vec![$($signature),*];
            for ele in &signature_elements {
                ctor_sig += ("L".to_string() + ele.replace(".", "/").as_str() + ";").as_str();
            }
            ctor_sig += ")V";
           let obj = env.new_object($instance_,ctor_sig, &args)?;
            Ok(
                $instance(obj.into())
            )
        }
    };
}
macro_rules! propagate_interface {
    ([$base:tt][$super_interface:tt][$concrete_type:ident] fn $fname:ident ($($arg:ident : $typ:ty),*)) => {
        fn $fname(&self, $($arg : $typ),*) -> Result<Box<dyn $super_interface + '_>> {
            let res = $base::$fname(self, $($arg),*)?.into();
            box_ok!($concrete_type(res))
        }
    }
}

macro_rules! box_ok {
    ($type:ident ($arg:ident)) => {
        Ok(Box::new($type($arg)))
    };
}