
package_name!("util");

use super::{
    units::{
        code::debug::{DebuggerVirtualMemory, IDebuggerVirtualMemory},
        IDexUnit, IUnit,
    },
    IRuntimeProject,
};
use crate::jeb::Instance;




pub trait IDecompilerUnit: Instance {
    fn decompileMethod(&self, identifier: &str) -> Result<bool>;
    fn getDecompiledMethodText(&self, msig: &str) -> Result<String>;
    fn decompileClass(&self, identifier: &str) -> Result<bool>;
    fn getDecompiledClassText(&self, csig: &str) -> Result<String>;
}

pub trait IDebuggerUnit: Instance {
    fn attach(
        &self,
        setup_info: Option<&setup_infos::DebuggerSetupInformation>,
    ) -> Result<bool>;
    fn getMemory(&self) -> Result<Box<dyn IDebuggerVirtualMemory + '_>>;
    fn detach(&self) -> Result<bool>;
    fn run(&self) -> Result<bool>;
    fn restart(&self) -> Result<bool>;
    fn clearBreakpoint(
        &self,
        break_point: Option<&dyn IDebuggerBreakpoint>,
    ) -> Result<bool>;
    fn clearBreakpoints(&self) -> Result<bool>;
    fn convertSymbolicAddressToMemoryToAddress(
        &self,
        symbol: String,
        unit: Option<&dyn IDexUnit>,
    ) -> Result<i64>;
    fn setBreakPoint(
        &self,
        address: i64,
    ) -> Result<Box<dyn IDebuggerBreakpoint + '_>>;
    fn setBreakPointWithSymbol(
        &self,
        address: &str,
        unit: Option<&dyn IDexUnit>,
    ) -> Result<Box<dyn IDebuggerBreakpoint + '_>>;
    fn addListener(
        &self,
        listener: Option<&dyn crate::jeb::debug_events::IDebugEventListener>,
    ) -> Result<()>;
    fn insertListener(
        &self,
        index : i32,
        listener: Option<&dyn crate::jeb::debug_events::IDebugEventListener>,
    ) -> Result<()>;

    fn pause(&self) -> Result<bool>;
    fn getThreadById(&self, id: i64) -> Result<Box<dyn IDebuggerThread + '_>>;
    fn getThreads(&self) -> Result<Vec<Box<dyn IDebuggerThread + '_>>>;
    fn isPaused(&self) -> Result<bool>;
}

pub trait IDebuggerBreakpoint: Instance {}
pub trait IDebuggerThread: Instance {
    fn getFrame(
        &self,
        index: i32,
    ) -> Result<Box<dyn IDebuggerThreadStackFrame + '_>>;
    fn getFrames(&self)
        -> Result<Vec<Box<dyn IDebuggerThreadStackFrame + '_>>>;
    fn stepInto(&self) -> Result<bool>;
    fn stepOut(&self) -> Result<bool>;
    fn stepOver(&self) -> Result<bool>;
    fn suspend(&self) -> Result<bool>;
    fn resume(&self) -> Result<bool>;
}
pub trait IDebuggerThreadStackFrame: Instance {
    fn getVariables(&self) -> Result<Vec<Box<dyn IDebuggerVariable + '_>>>;
    fn setVariable(&self, var: Option<&dyn IDebuggerVariable>) -> Result<bool>;
}
pub trait IDebuggerVariable: Instance {
    fn getName(&self) -> Result<String>;
    fn format(&self) -> Result<String>;
    fn getTypedValue(&self) -> Result<Box<dyn ITypedValue + '_>>;
    fn setTypeHint(&self, hint: &str) -> Result<bool>;
    fn setTypedValue(&self, val: Option<&dyn ITypedValue>) -> Result<bool>;
    fn canEditType(&self) -> Result<bool>;
    fn canEditValue(&self) -> Result<bool>;
    fn getAlternateName(&self) -> Result<String>;
    fn getFlags(&self) -> Result<i32>;
}
pub trait ITypedValueMarker: Instance {}
pub trait ITypedValue: Instance {
    fn format(&self) -> Result<String>;
    fn getTypeName(&self) -> Result<String>;
    fn getValue(&self) -> Result<jni::objects::JObject>;
}
impl<'a, T> ITypedValue for T
where
    T: 'a + Instance + ITypedValueMarker,
{
    fn format(&self) -> Result<String> {
        call!([String]self, "format", "()Ljava/lang/String;", &[])
    }

    fn getTypeName(&self) -> Result<String> {
        call!([String]self, "getTypeName", "()Ljava/lang/String;", &[])
    }

    fn getValue(&self) -> Result<jni::objects::JObject> {
        let res = call!(self, "getValue", "()Ljava/lang/Object;", &[])?;
        Ok(res.l()?)
    }
}

jclass! {JebDecompilerUnit, JebDecompilerUnit_}
jclass! {DecompilerHelper, DecompilerHelper_}

jclass! {JebDebuggerUnit, JebDebuggerUnit_}
jclass! {DebuggerHelper, DebuggerHelper_}
jclass! {DebuggerBreakPoint, DebuggerBreakPoint_}
jclass! {DebuggerThread, DebuggerThread_}
jclass! {DebuggerThreadStackFrame, DebuggerThreadStackFrame_ }
jclass! {DebuggerVariable, DebuggerVariable_}
jclass! {TypedVariable,TypedVariable_}

pub mod setup_infos {
    static PACKAGE_NAME: &str =
        "com/pnfsoftware/jeb/core/units/code/debug/impl";
    use crate::jeb::com::pnfsoftware::jeb::core::units::code::debug::{
        IDebuggerMachineInformation, IDebuggerProcessInformation,
    };
    use crate::jeb::*;

    jclass! {DebuggerSetupInformation,DebuggerSetupInformation_}
    impl<'a> DebuggerSetupInformation<'a> {
        pub fn create<'t>(
            hostname: String,
            port: i32,
        ) -> Result<'t, DebuggerSetupInformation<'t>>
        {
            let env = get_vm!();
            let res = env.call_static_method(
                DebuggerSetupInformation_,
                "create",
                normalize!("(Ljava.lang.String;I)Lcom.pnfsoftware.jeb.core.units.code.debug.impl.DebuggerSetupInformation;"),
                &[jstring!(hostname), port.into()],
            )?;
            Ok(DebuggerSetupInformation(res))
        }

        pub fn createWithMachine<'t>(
            machine: Option<&dyn IDebuggerMachineInformation>,
            process: Option<&dyn IDebuggerProcessInformation>,
        ) -> Result<'t, DebuggerSetupInformation<'t>>
        {
            let env = get_vm!();
            let args = jargs!(machine, process);
            let res = env.call_static_method(
                DebuggerSetupInformation_,
                "create",
                normalize!("(Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerMachineInformation;Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerProcessInformation;)Lcom.pnfsoftware.jeb.core.units.code.debug.impl.DebuggerSetupInformation;"),
                &args,
            )?;
            Ok(DebuggerSetupInformation(res))
        }
    }
}

impl<'a> DecompilerHelper<'a> {
    pub fn getDecompiler(
        unit: Option<&'a dyn IUnit>,
    ) -> Result<'a, Box<dyn IDecompilerUnit>> {
        let args = jargs!(unit);
        let env = get_vm!();

        let res = env.call_static_method(DecompilerHelper_, "getDecompiler", normalize!("(Lcom.pnfsoftware.jeb.core.units.IUnit;)Lcom.pnfsoftware.jeb.core.units.code.IDecompilerUnit;"), &args)?;
        Ok(Box::new(JebDecompilerUnit(res)))
    }
}

impl<'a> DebuggerHelper<'a> {
    pub fn getDebuggerForUnit(
        project: Option<&'a dyn IRuntimeProject>,
        unit: Option<&'a dyn IUnit>,
    ) -> Result<'a, Box<dyn IDebuggerUnit + 'a>> {
        let args = jargs!(project, unit);
        let env = get_vm!();

        let res = env.call_static_method(DebuggerHelper_, "getDebuggerForUnit", normalize!("(Lcom.pnfsoftware.jeb.core.IRuntimeProject;Lcom.pnfsoftware.jeb.core.units.code.ICodeUnit;)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerUnit;"), &args)?;
        if let Ok(obj) = res.l() {
            if env.is_same_object(obj, jni::objects::JObject::null())? {
                return Err("object is null".into());
            }
        }
        Ok(Box::new(JebDebuggerUnit(res)))
    }
}

impl<'a, 'b> TryFrom<&'b dyn IUnit<'a>> for JebDebuggerUnit<'b> {
    type Error = Box<dyn std::error::Error>;

    fn try_from<'t>(
        value: &'b dyn IUnit<'a>,
    ) -> core::result::Result<Self, Self::Error> {
        let env = get_vm!();
        let class = env.find_class(normalize!(
            "com.pnfsoftware.jeb.core.units.code.debug.IDebuggerUnit"
        ))?;
        if env.is_instance_of(value.get_obj()?, class)? {
            Ok(JebDebuggerUnit(value.get_obj()?.into()))
        } else {
            Err("Not an instance of IDebuggerUnit".into())
        }
    }
}
impl<'a> ITypedValue for TypedVariable<'a> {
    fn format(&self) -> Result<String> {
        call!([String]self, "format", "()Ljava/lang/String;", &[])
    }

    fn getTypeName(&self) -> Result<String> {
        call!([String]self, "getTypeName", "()Ljava/lang/String;", &[])
    }

    fn getValue(&self) -> Result<jni::objects::JObject> {
        Ok(call!(self, "getValue", "()Ljava/lang/Object;", &[])?.l()?)
    }
}

impl<'a> IDebuggerVariable for DebuggerVariable<'a> {
    fn getName(&self) -> Result<String> {
        call!([String]self, "getName", "()Ljava/lang/String;", &[])
    }

    fn format(&self) -> Result<String> {
        call!([String]self, "format", "()Ljava/lang/String;", &[])
    }
    fn setTypeHint(&self, hint: &str) -> Result<bool> {
        let args = vec![jstring! {hint}];
        call!([Bool]self, "setTypeHint", "(Ljava/lang/String;)Z", &args)
    }
    fn getTypedValue(&self) -> Result<Box<dyn ITypedValue + '_>> {
        let res = call!(
            self,
            "getTypedValue",
            normalize!(
                "()Lcom.pnfsoftware.jeb.core.units.code.debug.ITypedValue;"
            ),
            &[]
        )?;
        box_ok!(TypedVariable(res))
    }
    fn setTypedValue(&self, val: Option<&dyn ITypedValue>) -> Result<bool> {
        let args = jargs! {val};
        call!([Bool]self, "setTypedValue", "(Lcom.pnfsoftware.jeb.core.units.code.debug.ITypedValue;)Z", &args)
    }

    fn canEditType(&self) -> Result<bool> {
        call!([Bool]self, "canEditType", "()Z", &[])
    }

    fn canEditValue(&self) -> Result<bool> {
        call!([Bool]self, "canEditValue", "()Z", &[])
    }

    fn getAlternateName(&self) -> Result<String> {
        call!([String]self, "getAlternateName", "()Ljava/lang/String;", &[])
    }

    fn getFlags(&self) -> Result<i32> {
        call!([i32]self, "getFlags", "()I", &[])
    }
}

impl<'a> IDebuggerThreadStackFrame for DebuggerThreadStackFrame<'a> {
    jcall! {
        Vec<IDebuggerVariable>
        [DebuggerVariable]
        fn getVariables()
    }

    fn setVariable(&self, var: Option<&dyn IDebuggerVariable>) -> Result<bool> {
        let args = jargs! {var};
        call!([Bool]self, "setVariable", normalize!("(Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerVariable;)Z"), &args)
    }
}

impl<'a> IDebuggerThread for DebuggerThread<'a> {
    fn suspend(&self) -> Result<bool> {
        call!([Bool]self, "suspend", "()Z", &[])
    }
    fn resume(&self) -> Result<bool> {
        call!([Bool]self, "resume", "()Z", &[])
    }
    fn getFrame(
        &self,
        index: i32,
    ) -> Result<Box<dyn IDebuggerThreadStackFrame + '_>>
    {
        let res = call!(self, "getFrame", "(I)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerThreadStackFrame;", &[index.into()])?;
        box_ok!(DebuggerThreadStackFrame(res))
    }

    jcall! {
        Vec<IDebuggerThreadStackFrame>
        [DebuggerThreadStackFrame]
        fn getFrames()
    }
    fn stepInto(&self) -> Result<bool> {
        call!([Bool]self, "stepInto", "()Z", &[])
    }
    fn stepOut(&self) -> Result<bool> {
        call!([Bool]self, "stepOut", "()Z", &[])
    }
    fn stepOver(&self) -> Result<bool> {
        call!([Bool]self, "stepOver", "()Z", &[])
    }
}

impl<'a> IDebuggerUnit for JebDebuggerUnit<'a> {
    fn attach(
        &self,
        setup_info: Option<&setup_infos::DebuggerSetupInformation>,
    ) -> Result<bool> {
        let args = jargs!(setup_info);
        call!([Bool]self, "attach", normalize!("(Lcom.pnfsoftware.jeb.core.units.code.debug.impl.DebuggerSetupInformation;)Z"), &args)
    }
    fn getMemory(&self) -> Result<Box<dyn IDebuggerVirtualMemory + '_>> {
        let res = call!(self,"getMemory", normalize!("()Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerVirtualMemory;"), &[])?;
        let env = get_vm!();
        if env.is_same_object(res.l()?, jni::objects::JObject::null())? {
            return Err("null pointer".into());
        }
        box_ok!(DebuggerVirtualMemory(res))
    }
    fn detach(&self) -> Result<bool> {
        call!([Bool]self, "detach", normalize!("()Z"), &[])
    }
    fn isPaused(&self) -> Result<bool> {
        call!([Bool]self, "isPaused", normalize!("()Z"), &[])
    }

    fn run(&self) -> Result<bool> {
        call!([Bool]self, "run", normalize!("()Z"), &[])
    }

    fn restart(&self) -> Result<bool> {
        call!([Bool]self, "restart", normalize!("()Z"), &[])
    }
    fn pause(&self) -> Result<bool> {
        call!([Bool]self, "pause", normalize!("()Z"), &[])
    }

    fn clearBreakpoint(
        &self,
        break_point: Option<&dyn IDebuggerBreakpoint>,
    ) -> Result<bool> {
        let args = jargs!(break_point);
        call!([Bool]self, "clearBreakpoint", normalize!("(Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerBreakpoint;)Z"), &args)
    }

    fn clearBreakpoints(&self) -> Result<bool> {
        call!([Bool]self, "clearBreakpoints", normalize!("()Z"), &[])
    }

    fn getThreadById(&self, id: i64) -> Result<Box<dyn IDebuggerThread + '_>> {
        let res = call!(
            self,
            "getThreadById",
            "(J)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerThread;",
            &[id.into()]
        )?;
        box_ok!(DebuggerThread(res))
    }
    jcall! {
        Vec<IDebuggerThread>
        [DebuggerThread]
        fn getThreads()
    }

    fn convertSymbolicAddressToMemoryToAddress(
        &self,
        symbol: String,
        unit: Option<&dyn IDexUnit>,
    ) -> Result<i64> {
        let mut args = jargs!(unit);
        args.insert(0, jstring!(symbol));
        call!([i64]self, "convertSymbolicAddressToMemoryToAddress", normalize!("(Ljava.lang.String;Lcom.pnfsoftware.jeb.core.units.code.ICodeUnit;)J"), &args)
    }

    fn setBreakPoint(
        &self,
        address: i64,
    ) -> Result<Box<dyn IDebuggerBreakpoint + '_>> {
        let res = call!(
            self,
            "setBreakpoint",
            normalize!("(J)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerBreakpoint;"),
            &[address.into()]
        )?;
        box_ok!(DebuggerBreakPoint(res))
    }

    fn setBreakPointWithSymbol(
        &self,
        address: &str,
        unit: Option<&dyn IDexUnit>,
    ) -> Result<Box<dyn IDebuggerBreakpoint + '_>> {
        let mut args = jargs!(unit);
        args.insert(0, jstring!(address));
        let res = call!(self, "setBreakpoint", normalize!("(Ljava.lang.String;Lcom.pnfsoftware.jeb.core.units.code.ICodeUnit;)Lcom.pnfsoftware.jeb.core.units.code.debug.IDebuggerBreakpoint;"), &args)?;
        box_ok!(DebuggerBreakPoint(res))
    }

    fn addListener(
        &self,
        listener: Option<&dyn crate::jeb::debug_events::IDebugEventListener>,
    ) -> Result<()> {
        let env = get_vm!();
        let args = jargs! {listener};
        env.call_method(
            self.0.l()?,
            "addListener",
            normalize!("(Lcom.pnfsoftware.jeb.util.events.IEventListener;)V"),
            &args,
        )?;

        Ok(())
    }
    fn insertListener(
        &self,
        index : i32,
        listener: Option<&dyn crate::jeb::debug_events::IDebugEventListener>,
    ) -> Result<()> {
        let env = get_vm!();
        let mut args = jargs! {listener};
        args.insert(0, index.into());
        env.call_method(
            self.0.l()?,
            "insertListener",
            normalize!("(ILcom.pnfsoftware.jeb.util.events.IEventListener;)V"),
            &args,
        )?;

        Ok(())
    }
}
impl<'a> IDebuggerBreakpoint for DebuggerBreakPoint<'a> {}

impl<'a> IDecompilerUnit for JebDecompilerUnit<'a> {
    fn decompileMethod(&self, identifier: &str) -> Result<bool> {
        let args = vec![jstring!(identifier)];
        call!([Bool]self,"decompileMethod", normalize!("(Ljava.lang.String;)Z"), &args)
    }

    fn getDecompiledMethodText(&self, msig: &str) -> Result<String> {
        let args = vec![jstring!(msig)];
        call!([String]self, "getDecompiledMethodText", normalize!("(Ljava.lang.String;)Ljava.lang.String;"), &args)
    }
    fn decompileClass(&self, identifier: &str) -> Result<bool> {
        let args = vec![jstring!(identifier)];
        call!([Bool]self,"decompileClass", normalize!("(Ljava.lang.String;)Z"), &args)
    }

    fn getDecompiledClassText(&self, csig: &str) -> Result<String> {
        let args = vec![jstring!(csig)];
        call!([String]self, "getDecompiledClassText", normalize!("(Ljava.lang.String;)Ljava.lang.String;"), &args)
    }
}