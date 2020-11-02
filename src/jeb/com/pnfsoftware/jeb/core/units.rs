package_name!("units");

use self::code::android::dex::DexPoolType;

use super::util::{IDebuggerUnit, JebDebuggerUnit};

pub mod code {
    package_name!("code");
    pub mod debug {
        package_name!("debug");
        use crate::jeb::com::pnfsoftware::jeb::core::util::{ITypedValue, TypedVariable};

        pub mod impl_ {
            package_name!("impl");
            use crate::jeb::com::pnfsoftware::jeb::core::util::{
                ITypedValue, ITypedValueMarker,
            };
            

            jclass! {ValueString, ValueString_}
            jclass! {ValueBoolean, ValueBoolean_}
            jclass! {ValueInteger, ValueInteger_}
            jclass! {ValueLong, ValueLong_}
            jclass! {ValueFloat, ValueFloat_}
            jclass! {ValueDouble, ValueDouble_}

            impl<'a> ITypedValueMarker for ValueString<'a> {}
            impl<'a> ITypedValueMarker for ValueInteger<'a> {}
            impl<'a> ITypedValueMarker for ValueLong<'a> {}
            impl<'a> ITypedValueMarker for ValueFloat<'a> {}
            impl<'a> ITypedValueMarker for ValueBoolean<'a> {}

            impl<'a> ValueLong<'a> {
                pub fn getValue(&self) -> Result<i64> {
                    let res = ITypedValue::getValue(self)?;
                    call_object!([i64]res, "longValue", "()J", &[])
                }
            }
            impl<'a> ValueInteger<'a> {
                pub fn getValue(&self) -> Result<i32> {
                    let res = ITypedValue::getValue(self)?;
                    call_object!([i32]res, "intValue", "()I", &[])
                }
            }

            impl<'a> From<&'a dyn ITypedValue> for ValueLong<'a> {
                fn from(value: &'a dyn ITypedValue) -> Self {
                    ValueLong(value.get_obj().unwrap().into())
                }
            }
            impl<'a> From<&'a dyn ITypedValue> for ValueInteger<'a> {
                fn from(value: &'a dyn ITypedValue) -> Self {
                    ValueInteger(value.get_obj().unwrap().into())
                }
            }

            impl<'a> ValueString<'a> {
                pub fn from_object<'t>(
                    val: String,
                    object_id: i64,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let long = env.new_object(
                        "java/lang/Long",
                        "(J)V",
                        &[object_id.into()],
                    )?;

                    let res = env
                        .new_object(
                            ValueString_,
                            "(Ljava/lang/String;Ljava/lang/Long;)V",
                            &[jstring!(val), long.into()],
                        )?
                        .into();
                    box_ok!(ValueString(res))
                }
                pub fn new<'t>(
                    val: String,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let res: jni::objects::JValue = env
                        .new_object(
                            ValueString_,
                            "(Ljava/lang/String;)V",
                            &[jstring! {val}],
                        )?
                        .into();
                    box_ok!(ValueString(res))
                }
            }

            impl<'a> ValueBoolean<'a> {
                pub fn new<'t>(
                    val: bool,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let res = env
                        .new_object(ValueBoolean_, "(Z)V", &[val.into()])?
                        .into();
                    box_ok!(ValueBoolean(res))
                }
            }
            impl<'a> ValueInteger<'a> {
                pub fn new<'t>(
                    val: i32,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let res = env
                        .new_object(ValueInteger_, "(I)V", &[val.into()])?
                        .into();
                    box_ok!(ValueInteger(res))
                }
            }
            impl<'a> ValueLong<'a> {
                pub fn new<'t>(
                    val: i64,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let res = env
                        .new_object(ValueLong_, "(J)V", &[val.into()])?
                        .into();
                    box_ok!(ValueLong(res))
                }
            }
            impl<'a> ValueFloat<'a> {
                pub fn new<'t>(
                    val: f32,
                ) -> Result<'t, Box<dyn ITypedValue + 't>>
                {
                    let env = get_vm!();
                    let res = env
                        .new_object(ValueFloat_, "(F)V", &[val.into()])?
                        .into();
                    box_ok!(ValueFloat(res))
                }
            }
        }

        pub trait IVirtualMemory<'a>: Instance {
            fn isValidAddress(&self, address: i64) -> Result<bool>;
        }
        pub trait IVirtualMemoryMarker<'a>: IVirtualMemory<'a> {}
        pub trait IDebuggerVirtualMemory<'a>:
            Instance + IVirtualMemory<'a>
        {
        }
        pub trait IDebuggerVirtualMemoryMarker<'a>:
            IDebuggerVirtualMemory<'a>
        {
        }
        pub trait IDebuggerUnitIdentifier: Instance {
            fn getTargetEnumerator(
                &self,
            ) -> Result<Box<dyn IDebuggerTargetEnumerator + '_>>;
        }
        pub trait IDebuggerTargetEnumerator: Instance {
            fn listMachines(
                &self,
            ) -> Result<Vec<Box<dyn IDebuggerMachineInformation + '_>>>;
        }
        pub trait IDebuggerEventData: Instance {
            fn getAddress(&self) -> Result<String>;
            fn getOutput(&self) -> Result<Vec<u8>>;
            fn getReturnValue(&self) -> Result<Box<dyn ITypedValue + '_>>;
            fn getThreadId(&self) -> Result<i64>;
            fn getType(&self) -> Result<DebuggerEventType>;
        }

        impl<'a> From<jni::objects::JObject<'a>> for DebuggerEventType {
            fn from(data: jni::objects::JObject<'a>) -> Self {
                let env = get_vm_unwrap!();
                if let Ok(res) = env.call_method(
                    data,
                    "toString",
                    "()Ljava/lang/String;",
                    &[],
                ) {
                    let string: String = env
                        .get_string(res.l().unwrap().into())
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    match string.as_str() {
                        "BREAKPOINT" => DebuggerEventType::Breakpoint,
                        "BREAKPOINT_FUNCTION_EXIT" => {
                            DebuggerEventType::BreakpointFunctionExit
                        }
                        "CODE_LOAD" => DebuggerEventType::CodeLoad,
                        "CODE_UNLOAD" => DebuggerEventType::CodeUnload,
                        "EXCEPTION" => DebuggerEventType::Exception,
                        "FUNCTION_ENTRY" => DebuggerEventType::FunctionEntry,
                        "FUNCTION_EXIT" => DebuggerEventType::FunctionExit,
                        "OUTPUT" => DebuggerEventType::Output,
                        "SIGNAL" => DebuggerEventType::Signal,
                        "SUSPENDED" => DebuggerEventType::Suspended,
                        "THREAD_START" => DebuggerEventType::ThreadStart,
                        "THREAD_STOP" => DebuggerEventType::ThreadStop,
                        _ => {
                            println!("{}", string);
                            DebuggerEventType::Unknown
                        }
                    }
                } else {
                    DebuggerEventType::Unknown
                }
            }
        }

        pub trait IDebuggerMachineInformation: Instance {
            fn getFlags(&self) -> Result<i32>;
            fn getInformation(&self) -> Result<String>;
            fn getLocation(&self) -> Result<String>;
            fn getName(&self) -> Result<String>;
            fn getProcesses(
                &self,
            ) -> Result<Vec<Box<dyn IDebuggerProcessInformation + '_>>>;
        }
        pub trait IDebuggerProcessInformation: Instance {
            fn getFlags(&self) -> Result<i32>;
            fn getId(&self) -> Result<i64>;
            fn getName(&self) -> Result<String>;
        }

        pub enum DebuggerEventType {
            Breakpoint,
            BreakpointFunctionExit,
            CodeLoad,
            CodeUnload,
            Exception,
            FunctionEntry,
            FunctionExit,
            Output,
            Signal,
            Suspended,
            ThreadStart,
            ThreadStop,
            Unknown,
        }

        jclass! {DebuggerVirtualMemory, DebuggerVirtualMemory_}
        impl<'a> IVirtualMemoryMarker<'a> for DebuggerVirtualMemory<'a> {}
        impl<'a> IDebuggerVirtualMemoryMarker<'a> for DebuggerVirtualMemory<'a> {}

        jclass! {DebuggerUnitIdentifier, DebuggerUnitIdentifier_}
        jclass! {DebuggerTargetEnumerator, DebuggerTargetEnumerator_}
        jclass! {DebuggerMachineInformation, DebuggerMachineInformation_}
        jclass! {DebuggerProcessInformation, DebuggerProcessInformation_}
        jclass! {DebuggerEventData, DebuggerEventData_}

        impl<'a, T> IVirtualMemory<'a> for T
        where
            T: 'a + IVirtualMemoryMarker<'a> + Instance,
        {
            fn isValidAddress(&self, address: i64) -> Result<bool> {
                call!([Bool]self, "isValidAddress", "(J)Z", &[address.into()])
            }
        }
        impl<'a, T> IDebuggerVirtualMemory<'a> for T where
            T: 'a + IDebuggerVirtualMemoryMarker<'a> + Instance
        {
        }

        use std::convert::TryInto;
        impl<'a> IDebuggerEventData for DebuggerEventData<'a> {
            fn getAddress(&self) -> Result<String> {
                call!([String]self, "getAddress", "()Ljava/lang/String;", &[])
            }

            fn getOutput(&self) -> Result<Vec<u8>> {
                let res = call!(self, "getOutput", "()[B", &[])?;
                if let Ok(ba) = res.l() {
                    let env = get_vm!();
                    let array = env.convert_byte_array(ba.into_inner())?;
                    Ok(array)
                } else {
                    Err("not an object".into())
                }
            }

            fn getReturnValue(&self) -> Result<Box<dyn ITypedValue + '_>> {
                let res = call!(self, "getReturnValue", "()Lcom.pnfsoftware.jeb.core.units.code.debug.ITypedValue;", &[])?;
                box_ok!(TypedVariable(res))
            }

            fn getThreadId(&self) -> Result<i64> {
                call!([i64]self, "getThreadId", "()J", &[])
            }

            fn getType(&self) -> Result<DebuggerEventType> {
                let res = call!(self, "getType", normalize!("()Lcom.pnfsoftware.jeb.core.units.code.debug.DebuggerEventType;"), &[])?;
                if let Ok(obj) = res.l() {
                    Ok(obj.try_into()?)
                } else {
                    Err("not an object".into())
                }
            }
        }

        impl<'a> TryFrom<jni::objects::JObject<'a>> for DebuggerEventData<'a> {
            type Error = Box<dyn std::error::Error>;

            fn try_from(
                value: jni::objects::JObject<'a>,
            ) -> core::result::Result<Self, Self::Error>
            {
                let env = get_vm!();
                let class = env.find_class(normalize!(
                    "com.pnfsoftware.jeb.core.units.code.debug.IDebuggerEventData"
                ))?;
                if env.is_instance_of(value, class)? {
                    Ok(DebuggerEventData(value.into()))
                } else {
                    Err("Is not a ClientNotification".into())
                }
            }
        }

        impl<'a> IDebuggerUnitIdentifier for DebuggerUnitIdentifier<'a> {
            fn getTargetEnumerator(
                &self,
            ) -> Result<Box<dyn IDebuggerTargetEnumerator + '_>>
            {
                let res = call!(self, "getTargetEnumerator", "()Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerTargetEnumerator;", &[])?;
                if let Ok(obj) = res.l() {
                    let env = get_vm!();
                    if env.is_same_object(obj, jni::objects::JObject::null())? {
                        Err("Object is null".into())
                    } else {
                        box_ok!(DebuggerTargetEnumerator(res))
                    }
                } else {
                    Err("Return value is not an object".into())
                }
            }
        }
        impl<'a> IDebuggerTargetEnumerator for DebuggerTargetEnumerator<'a> {
            jcall! {
                Vec<IDebuggerMachineInformation>
                [DebuggerMachineInformation]
                fn listMachines()
            }
        }
        impl<'a> IDebuggerMachineInformation for DebuggerMachineInformation<'a> {
            fn getFlags(&self) -> Result<i32> {
                call!([i32]self, "getFlags", "()I", &[])
            }

            fn getInformation(&self) -> Result<String> {
                call!([String]self, "getInformation", "()Ljava/lang/String;", &[])
            }

            fn getLocation(&self) -> Result<String> {
                call!([String]self, "getLocation", "()Ljava/lang/String;", &[])
            }

            fn getName(&self) -> Result<String> {
                call!([String]self, "getName", "()Ljava/lang/String;", &[])
            }

            jcall! {
                Vec<IDebuggerProcessInformation>
                [DebuggerProcessInformation]
                fn getProcesses()
            }
        }
        impl<'a> IDebuggerProcessInformation for DebuggerProcessInformation<'a> {
            fn getFlags(&self) -> Result<i32> {
                call!([i32]self, "getFlags", "()I", &[])
            }

            fn getId(&self) -> Result<i64> {
                call!([i64]self, "getId", "()J", &[])
            }

            fn getName(&self) -> Result<String> {
                call!([String]self, "getName", "()Ljava/lang/String;", &[])
            }
        }
    }
    pub mod android {
        pub mod dex {
            static PACKAGE_NAME: &str =
                "com/pnfsoftware/jeb/core/units/code/android/dex";

            use crate::jeb::*;

            pub enum DexPoolType {
                CallSite,
                Class,
                Field,
                Method,
                MethodHandle,
                Prototype,
                String,
                Type,
            }

            impl<'a> From<&'a DexPoolType> for jni::objects::JValue<'a> {
                fn from(pool_type: &'a DexPoolType) -> Self {
                    match pool_type {
                        DexPoolType::CallSite => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "CALL_SITE", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::Class => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "CLASS", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::Field => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "FIELD", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::Method => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "METHOD", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::MethodHandle => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "METHOD_HANDLE", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::Prototype => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "PROTOTYPE", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::String => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "STRING", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                        DexPoolType::Type => {
                            let env = get_vm_unwrap!();
                            env.get_static_field(normalize!("com.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType"), 
                                                "TYPE", 
                                                normalize!("Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;")).unwrap()
                        }
                    }
                }
            }
        }
    }
}

pub trait IUnit<'a>: Instance {
    fn toString(&self) -> Result<String>;
    fn getChildren(&self) -> Result<Vec<Box<dyn IUnit + '_>>>;
    fn getDescription(&self) -> Result<String>;
    fn getFormatType(&self) -> Result<String>;
    fn getName(&self) -> Result<String>;
    fn getStatus(&self) -> Result<String>;
    fn isProcessed(&self) -> Result<bool>;
    fn process(&self) -> Result<()>;
    fn addListener(
        &self,
        listener: Option<&dyn INativeEventListener>,
    ) -> Result<()>;
    fn getUnitProcessor(&self) -> Result<Box<dyn IUnitProcessor + '_>>;
}
pub trait IUnitMarker<'a>: IUnit<'a> {}
pub trait IUnitProcessor<'a>: Instance {
    fn createDebugger(
        &self,
        name: &str,
        parent: Option<&dyn IUnit>,
    ) -> Result<Box<dyn IDebuggerUnit + '_>>;
}
pub trait IUnitProcessorMarker {}
pub trait IDexPackage<'a>: Instance {
    fn getName(&self) -> Result<String>;
    fn getParentPackage(&mut self) -> Result<()>;
    fn getChildrenPackages(&self) -> Result<Vec<Box<dyn IDexPackage + '_>>>;
    fn isRootPackage(&self) -> Result<bool>;
}
pub trait IDexPackageMarker<'a>: IDexPackage<'a> {}
pub trait IDexMethod<'a>: Instance {
    fn getSignature(&self, effective: bool) -> Result<String>;
    fn getIndex(&self) -> Result<i32>;
    fn getName(&self, effective: bool) -> Result<String>;
    fn getInstructions(&self) -> Result<Vec<Box<dyn IInstruction + '_>>>;
}
pub trait IDexMethodMarker<'a>: IDexMethod<'a> {}

pub trait IInstruction {
    fn format(&self, context: Option<&dyn IUnit>) -> Result<String>;
    fn getSize(&self) -> Result<i32>;
    fn getOffset(&self) -> Result<i64>;
}
pub trait IInstructionMarker {}

pub trait IDexClass<'a>: Instance {
    fn getMethods(&self) -> Result<Vec<Box<dyn IDexMethod + '_>>>;
    fn getName(&self) -> Result<String>;
    fn getPackage(&self) -> Result<Box<dyn IDexPackage + '_>>;
    fn getSignature(&self) -> Result<String>;
    fn getIndex(&self) -> Result<i32>;
}
pub trait IDexClassMarker<'a>: IDexClass<'a> {}
pub trait IDexString<'a>: Instance {
    fn getIndex(&self) -> Result<i32>;
    fn getName(&self, effective: bool) -> Result<String>;
    fn getIdentifier(&self) -> Result<i64>;
    fn getValue(&self) -> Result<String>;
}
pub trait IDexStringMarker<'a>: IDexString<'a> {}

pub trait IDexUnit<'a>: Instance {
    fn getDisassembly(&self) -> Result<String>;
    fn getMethodByIndex(&self, idx: i32) -> Result<Box<dyn IDexMethod + '_>>;
    fn getMethodByName(&self, fqname: &str)
        -> Result<Box<dyn IDexMethod + '_>>;
    fn getMethods(&self) -> Result<Vec<Box<dyn IDexMethod + '_>>>;
    fn getClasses(&self) -> Result<Vec<Box<dyn IDexClass + '_>>>;
    fn getReferenceManager(&self)
        -> Result<Box<dyn IDexReferenceManager + '_>>;
    fn getString(&self, idx: i32) -> Result<Box<dyn IDexString + '_>>;
    fn getStrings(&self) -> Result<Vec<Box<dyn IDexString + '_>>>;
}
pub trait IDexUnitMarker<'a>: IDexUnit<'a> {}

pub trait IDexReferenceManager<'a>: Instance {
    fn getReferences(
        &self,
        pool_type: &DexPoolType,
        index: i32,
    ) -> Result<Vec<Box<dyn IDexAddress + '_>>>;
}

pub trait IDexAddress<'a>: Instance {
    fn getInternalAddress(&self) -> Result<String>;
}

impl<'a> IDexPackage<'a> for JebDexPackage<'a> {
    fn getName(&self) -> Result<String> {
        call!([String]self, "getName", normalize!("()Ljava.lang.String;"), &[])
    }
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebDexPackage]
        fn getChildrenPackages() -> Vec<Box<dyn IDexPackage + '_>> {
            vec![]
        }
    }
    fn getParentPackage(&mut self) -> Result<()> {
        let res = call_object!(
            self.0.l()?,
            "getParentPackage",
            "()Lcom.pnfsoftware.jeb.core.units.code.ICodePackage;",
            &[]
        )?;
        self.0 = res;
        Ok(())
    }
    fn isRootPackage(&self) -> Result<bool> {
        call!([Bool]self, "isRootPackage", normalize!("()Z"), &[])
    }
}

impl<'a, T> IDexUnit<'a> for T
where
    T: 'a + IDexUnitMarker<'a> + IUnitMarker<'a> + Instance,
{
    fn getDisassembly(&self) -> Result<String> {
        call!([String]self, "getDisassembly", "()Ljava/lang/String;", &[])
    }
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebDexMethod]
        fn getMethods() -> Vec<Box<dyn IDexMethod + '_>> {
            vec![]
        }
    }
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebDexClass]
        fn getClasses() -> Vec<Box<dyn IDexClass + '_>> {
            vec![]
        }
    }
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebDexString]
        fn getStrings() -> Vec<Box<dyn IDexString + '_>> {
            vec![]
        }
    }

    fn getReferenceManager(
        &self,
    ) -> Result<Box<dyn IDexReferenceManager + '_>> {
        let res = call!(self, "getReferenceManager", normalize!("()Lcom.pnfsoftware.jeb.core.units.code.android.IDexReferenceManager;"), &[])?;
        Ok(Box::new(JebDexReferenceManager(res)))
    }
    fn getString(&self, idx: i32) -> Result<Box<dyn IDexString + '_>> {
        let args = [idx.into()];
        let res = call!(self, "getString", normalize!("(I)Lcom.pnfsoftware.jeb.core.units.code.android.dex.IDexString;"), &args)?;
        Ok(Box::new(JebDexString(res)))
    }
    fn getMethodByIndex(&self, idx: i32) -> Result<Box<dyn IDexMethod + '_>> {
        let args = [idx.into()];
        let res = call!(self, "getMethod", normalize!("(I)Lcom.pnfsoftware.jeb.core.units.code.android.dex.IDexMethod;"), &args)?;
        Ok(Box::new(JebDexMethod(res)))
    }
    fn getMethodByName(
        &self,
        fqname: &str,
    ) -> Result<Box<dyn IDexMethod + '_>> {
        let args = [jstring! {fqname}];
        let res = call!(self, "getMethod", normalize!("(Ljava.lang.String;)Lcom.pnfsoftware.jeb.core.units.code.android.dex.IDexMethod;"), &args)?;
        Ok(Box::new(JebDexMethod(res)))
    }
}
impl<'a, T> IDexClass<'a> for T
where
    T: 'a + IDexClassMarker<'a> + Instance,
{
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebDexMethod]
        fn getMethods() -> Vec<Box<dyn IDexMethod + '_>> {
            vec![]
        }
    }
    fn getIndex(&self) -> Result<i32> {
        call!([i32]self, "getIndex", normalize!("()I"), &[])
    }
    fn getName(&self) -> Result<String> {
        call!([String]self, "getName", normalize!("()Ljava.lang.String;"), &[])
    }
    fn getSignature(&self) -> Result<String> {
        call!([String]self, "getSignature", normalize!("()Ljava.lang.String;"), &[])
    }
    fn getPackage(&self) -> Result<Box<dyn IDexPackage + '_>> {
        let res = call!(
            self,
            "getPackage",
            "()Lcom.pnfsoftware.jeb.core.units.code.ICodePackage;",
            &[]
        )?;
        Ok(Box::new(JebDexPackage(res)))
    }
}

impl<'a, T> IDexMethod<'a> for T
where
    T: 'a + IDexMethodMarker<'a> + Instance,
{
    fn getSignature(&self, effective: bool) -> Result<String> {
        let args = [effective.into()];
        call!([String]self, "getSignature", normalize!("(Z)Ljava.lang.String;"), &args)
    }
    fn getIndex(&self) -> Result<i32> {
        call!([i32]self, "getIndex", normalize!("()I"), &[])
    }
    fn getName(&self, effective: bool) -> Result<String> {
        call!([String]self, "getName", normalize!("(Z)Ljava.lang.String;"), &[effective.into()])
    }
    jcall! {
        Vec<IInstruction>
        [JebInstruction]
        fn getInstructions()
    }
}

impl<'a> IInstruction for JebInstruction<'a> {
    fn format(&self, context: Option<&dyn IUnit>) -> Result<String> {
        let args = jargs!(context);
        call!([String]self, "format", "(Ljava/lang/Object;)Ljava/lang/String;", &args)
    }
    fn getSize(&self) -> Result<i32> {
        call!([i32]self, "getSize", "()I", &[])
    }
    fn getOffset(&self) -> Result<i64> {
        call!([i64]self, "getOffset", "()J", &[])
    }
}

impl<'a, 'b> TryFrom<&'b dyn IUnit<'a>> for JebDexUnit<'b> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: &'b dyn IUnit<'a>,
    ) -> core::result::Result<Self, Self::Error> {
        if value.getFormatType()? == "dex" {
            Ok(JebDexUnit(value.get_obj()?.into()))
        } else {
            Err("Not a DexUnit".into())
        }
    }
}

jclass! {JebInstruction, JebInstruction_}
jclass! {JebUnit, JebUnit_}
impl<'a> IUnitMarker<'a> for JebUnit<'a> {}

jclass! {JebUnitProcessor, JebUnitProcessor_}
jclass! {JebDexClass,JebDexClass_}
impl<'a> IDexClassMarker<'a> for JebDexClass<'a> {}
jclass! {JebDexUnit, JebDexUnit_}
impl<'a> IDexUnitMarker<'a> for JebDexUnit<'a> {}
impl<'a> IUnitMarker<'a> for JebDexUnit<'a> {}

jclass! {JebDexMethod, JebDexMethod_}
impl<'a> IDexMethodMarker<'a> for JebDexMethod<'a> {}
jclass! {JebDexReferenceManager,JebDexReferenceManager_}
jclass! {JebDexPackage, JebDexPackage_}
jclass! {AbstractUnit,AbstractUnit_}
jclass! {AbstractCodeUnit,AbstractCodeUnit_}
jclass! {JebDexAddress, JebDexAddress_}
jclass! {JebDexString,JebDexString_}

impl<'a> IDexAddress<'a> for JebDexAddress<'a> {
    fn getInternalAddress(&self) -> Result<String> {
        call!([String]self, "getInternalAddress", "()Ljava/lang/String;", &[])
    }
}
impl<'a> IDexString<'a> for JebDexString<'a> {
    fn getIndex(&self) -> Result<i32> {
        call!([i32]self, "getIndex", "()I", &[])
    }

    fn getName(&self, effective: bool) -> Result<String> {
        call!([String]self, "getName", normalize!("(Z)Ljava.lang.String;"), &[effective.into()])
    }

    fn getIdentifier(&self) -> Result<i64> {
        call!([i64]self, "getIndex", "()L", &[])
    }

    fn getValue(&self) -> Result<String> {
        call!([String]self, "getValue", normalize!("()Ljava.lang.String;"), &[])
    }
}

impl<'a> IDexReferenceManager<'a> for JebDexReferenceManager<'a> {
    jcall! {
        Vec[normalize!("(Lcom.pnfsoftware.jeb.core.units.code.android.dex.DexPoolType;I)Ljava/util/Collection;")]
        [JebDexAddress]
        fn getReferences(pool_type : &DexPoolType, index : i32) -> Vec<Box<dyn IDexAddress + '_>> {
            vec![pool_type.into(), index.into()]
        }
    }
}

impl<'a> IUnitProcessor<'a> for JebUnitProcessor<'a> {
    fn createDebugger(
        &self,
        name: &str,
        parent: Option<&dyn IUnit>,
    ) -> Result<Box<dyn IDebuggerUnit + '_>> {
        let mut args = jargs!(parent);
        args.insert(0, jstring! {name});
        let res = call!(self, "createDebugger", normalize!("(Ljava.lang.String;Lcom.pnfsoftware.jeb.core.units.IUnit;)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerUnit;"), &args)?;
        let env = get_vm!();
        if env.is_same_object(res.l()?, jni::objects::JObject::null())? {
            Err("Object is null".into())
        } else {
            box_ok!(JebDebuggerUnit(res))
        }
    }
}

impl<'a, T> IUnit<'a> for T
where
    T: 'a + IUnitMarker<'a> + Instance,
{
    fn getUnitProcessor(&self) -> Result<Box<dyn IUnitProcessor + '_>> {
        let res = call!(
            self,
            "getUnitProcessor",
            "()Lcom.pnfsoftware.jeb.core.units.IUnitProcessor;",
            &[]
        )?;
        box_ok!(JebUnitProcessor(res))
    }
    fn toString(&self) -> Result<String> {
        let env = get_vm!();
        let obj = self.get_obj()?;
        let the_string =
            env.call_method(obj, "toString", "()Ljava/lang/String;", &[])?;
        if let jni::objects::JValue::Object(the_string) = the_string {
            Ok(env.get_string(the_string.into())?.into())
        } else {
            Err("no string".into())
        }
    }
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [JebUnit]
        fn getChildren() -> Vec<Box<dyn IUnit + '_>> {
            vec![]
        }
    }
    fn addListener(
        &self,
        listener: Option<&dyn INativeEventListener>,
    ) -> Result<()> {
        let env = get_vm!();
        let args = jargs! {listener};
        env.call_method(
            self.get_obj()?,
            "addListener",
            normalize!("(Lcom.pnfsoftware.jeb.util.events.IEventListener;)V"),
            &args,
        )?;

        Ok(())
    }
    fn getDescription(&self) -> Result<String> {
        call!([String] self, "getDescription", "()Ljava.lang.String;", &[])
    }
    fn getFormatType(&self) -> Result<String> {
        call!([String] self, "getFormatType", "()Ljava.lang.String;", &[])
    }
    fn getName(&self) -> Result<String> {
        call!([String] self, "getName", "()Ljava.lang.String;", &[])
    }
    fn getStatus(&self) -> Result<String> {
        call!([String] self, "getStatus", "()Ljava.lang.String;", &[])
    }

    fn isProcessed(&self) -> Result<bool> {
        call!([Bool] self, "isProcessed", "()Z", &[])
    }
    fn process(&self) -> Result<()> {
        call!(self, "process", "()V", &[])?;
        Ok(())
    }
}