#[macro_use]
use crate::jeb::helper_macros::*;
package_name!("core");

pub mod events;
pub mod util;
pub mod properties;
pub mod dao;
pub mod units;
pub mod input;

use crate::jeb::*;

use self::units::code::debug::{DebuggerUnitIdentifier, IDebuggerUnitIdentifier};


pub trait IArtifact<'a>: Instance {}
pub trait ICoreContext<'a>: Instance {
    fn createEnginesContext(
        &self,
        data_provider: Option<&dyn dao::IDataProvider>,
        client_information: Option<&JebClientInformation>,
    ) -> Result<Box<dyn IEnginesContext + '_>>;
    fn closeEnginesContext(
        &self,
        context: Option<&dyn IEnginesContext>,
    ) -> Result<()>;
}
pub trait IEnginesContext<'a>: Instance {
    fn loadProject(&self, str: &str) -> Result<Box<dyn IRuntimeProject + '_>>;
    fn unloadProject(&self, key: &str) -> Result<bool>;
    fn getDebuggerUnitIdentifiers(
        &self,
    ) -> Result<Vec<Box<dyn IDebuggerUnitIdentifier + '_>>>;
    fn isIdentifierEnabled(
        &self,
        identifier: Option<&dyn IDebuggerUnitIdentifier>,
    ) -> Result<bool>;
    fn setIdentifierEnabled(
        &self,
        identifier: Option<&dyn IDebuggerUnitIdentifier>,
        enabled: bool,
    ) -> Result<bool>;
}

pub trait ILiveArtifact<'a>: Instance {
    fn getUnits(&self) -> Result<Vec<Box<dyn units::IUnit + '_>>>;
}
pub trait IRuntimeProject<'a>: Instance {
    fn processArtifact(
        &self,
        artifact: Option<&dyn IArtifact>,
    ) -> Result<Box<dyn ILiveArtifact + '_>>;
    fn getKey(&self) -> Result<String>;
}

jclass! {RuntimeProjectUtil, RuntimeProjectUtil_}
jclass! {Artifact, Artifact_}
jclass! {JebLiveArtifact, JebLiveArtifact_}
jclass! {JebCoreService, JebCoreService_}
jclass! {JebClientInformation, JebClientInformation_}
jclass! {JebEnginesContext, JebEnginesContext_}
jclass! {JebRuntimeProject, JebRuntimeProject_}

impl<'a> RuntimeProjectUtil<'a> {
    pub fn getAllUnits<'t>(
        prj: Option<&'_ dyn IRuntimeProject>,
    ) -> Result<'t, Vec<Box<dyn units::IUnit<'t> + 't>>> {
        let env = get_vm!();
        let args = jargs! {prj};
        let res = env.call_static_method(
            RuntimeProjectUtil_,
            "getAllUnits",
            normalize!(
                "(Lcom.pnfsoftware.jeb.core.IRuntimeProject;)Ljava/util/List;"
            ),
            &args,
        )?;
        if let jni::objects::JValue::Object(array) = res {
            let list = jni::objects::JList::from_env(&env, array)?;
            let mut result: Vec<Box<dyn units::IUnit>> = vec![];
            for element in list.iter()? {
                result.push(Box::new(units::JebUnit(element.into())))
            }
            Ok(result)
        } else {
            Err("return type is not a object".into())
        }
    }
    pub fn findUnitsByType<'t>(
        prj: Option<&'_ dyn IRuntimeProject>,
        ty: &str,
        strict: bool,
    ) -> Result<'t, Vec<Box<dyn units::IUnit<'t> + 't>>> {
        let env = get_vm!();
        let class = env.find_class(normalize!(ty))?;
        let mut args = jargs! {prj};
        args.push(class.into());
        args.push(strict.into());
        let res = env.call_static_method(
            RuntimeProjectUtil_,
            "findUnitsByType",
            normalize!(
                "(Lcom.pnfsoftware.jeb.core.IRuntimeProject;Ljava.lang.Class;Z)Ljava/util/List;"
            ),
            &args
        )?;
        if let jni::objects::JValue::Object(array) = res {
            let list = jni::objects::JList::from_env(&env, array)?;
            let mut result: Vec<Box<dyn units::IUnit>> = vec![];

            for element in list.iter()? {
                result.push(Box::new(units::JebUnit(element.into())))
            }
            Ok(result)
        } else {
            Err("return type is not a object".into())
        }
    }
}

impl<'a> Artifact<'a> {
    pub fn new<'t>(file_name: String) -> Result<'t, Box<dyn IArtifact<'t> + 't>> {
        let env = get_vm!();
        let file_borrow = file_name.as_str();
        let args = vec![jstring!(file_borrow)];
        let obj = env.new_object("java/io/File", "(Ljava/lang/String;)V", &args)?;

        let file = file::File(obj.into());
        let file_input = Some(input::FileInput::from_file(Some(file))?);
        let ctor_sig = format!(
            "(L{};L{};)V",
            "java/lang/String",
            normalize!(
                "com.pnfsoftware.jeb.core.input.IInput
        "
            )
        );

        let mut args = jargs! {file_input};

        args.insert(0, jstring! {file_borrow});
        let res = env.new_object(Artifact_, ctor_sig, &args)?;

        Ok(Box::new(Artifact(res.into())))
    }
}

impl<'a> IArtifact<'a> for Artifact<'a> {}

impl<'a> JebCoreService<'a> {
    pub fn getInstance<'t>(
        license_key: &'t str,
    ) -> Result<Box<dyn ICoreContext + 't>> {
        let env = VM.attach_current_thread_permanently()?;
        let license_key: jni::objects::JString = env.new_string(license_key)?;
        let args: Vec<jni::objects::JValue> = vec![license_key.into()];
        let res = env.call_static_method(
            JebCoreService_,
            "getInstance",
            "(Ljava/lang/String;)Lcom/pnfsoftware/jeb/core/ICoreContext;",
            &args,
        )?;

        Ok(Box::new(JebCoreService(res)))
    }
    pub fn getOldInstance<'t>() -> Result<'t, Box<dyn ICoreContext<'t> + 't>> {
        let env = VM.attach_current_thread_permanently()?;
        let res = env.call_static_method(
            JebCoreService_,
            "getInstance",
            "()Lcom/pnfsoftware/jeb/core/ICoreContext;",
            &[],
        )?;

        Ok(Box::new(JebCoreService(res)))
    }
}

impl<'a> ILiveArtifact<'a> for JebLiveArtifact<'a> {
    jcall! {
        Vec[normalize!("()Ljava/util/List;")]
        [units::JebUnit]
        fn getUnits() -> Vec<Box<dyn units::IUnit + '_>> {
            vec![]
        }
    }
}

impl<'a> IRuntimeProject<'a> for JebRuntimeProject<'a> {
    fn processArtifact(
        &self,
        artifact: Option<&dyn IArtifact>,
    ) -> Result<Box<dyn ILiveArtifact + '_>> {
        let env = get_vm!();
        let args = jargs!(artifact);

        let res = env.call_method(self.0.l()?, "processArtifact", normalize!("(Lcom.pnfsoftware.jeb.core.IArtifact;)Lcom.pnfsoftware.jeb.core.ILiveArtifact;"), &args)?;
        Ok(Box::new(JebLiveArtifact(res)))
    }
    fn getKey(&self) -> Result<String> {
        let env = get_vm!();
        let res = call!(self, "getKey", "()Ljava/lang/String;", &[])?;
        let obj = res.l()?;
        let the_string = env.get_string(obj.into())?;
        Ok(the_string.into())
    }
}

impl<'a> IEnginesContext<'a> for JebEnginesContext<'a> {
    fn loadProject(&self, key: &'_ str) -> Result<Box<dyn IRuntimeProject + '_>> {
        let args = vec![jstring!(key)];
        let env = get_vm!();

        let res = env.call_method(
            self.0.l()?,
            "loadProject",
            normalize!(
                "(Ljava/lang/String;)Lcom.pnfsoftware.jeb.core.IRuntimeProject;"
            ),
            &args,
        )?;
        Ok(Box::new(JebRuntimeProject(res)))
    }
    fn unloadProject(&self, key: &str) -> Result<bool> {
        let args = vec![jstring!(key)];
        let res = call! {self, "unloadProject", "(Ljava/lang/String;)Z", &args}?;
        Ok(res.z()?)
    }

    jcall! {
        Vec<IDebuggerUnitIdentifier>
        [DebuggerUnitIdentifier]
        fn getDebuggerUnitIdentifiers()
    }
    fn isIdentifierEnabled(
        &self,
        identifier: Option<&dyn IDebuggerUnitIdentifier>,
    ) -> Result<bool> {
        let args = jargs!(identifier);
        call!([Bool]self, "isIdentifierEnabled", normalize!("(Lcom.pnfsoftware.jeb.core.units.IUnitIdentifier;)Z"), &args)
    }
    fn setIdentifierEnabled(
        &self,
        identifier: Option<&dyn IDebuggerUnitIdentifier>,
        enabled: bool,
    ) -> Result<bool> {
        let mut args = jargs!(identifier);
        args.push(enabled.into());
        call!([Bool]self, "setIdentifierEnabled", normalize!("(Lcom.pnfsoftware.jeb.core.units.IUnitIdentifier;Z)Z"), &args)
    }
}

impl<'a> JebClientInformation<'a> {}

impl<'a> ICoreContext<'a> for JebCoreService<'a> {
    fn createEnginesContext(
        &self,
        data_provider: Option<&dyn dao::IDataProvider>,
        client_information: Option<&JebClientInformation>,
    ) -> Result<Box<dyn IEnginesContext + '_>> {
        let args = jargs!(data_provider, client_information);
        let env = get_vm!();

        let res = env.call_method(self.0.l()?, "createEnginesContext", normalize!("(Lcom.pnfsoftware.jeb.core.dao.IDataProvider;Lcom.pnfsoftware.jeb.core.JebClientInformation;)Lcom.pnfsoftware.jeb.core.IEnginesContext;"), &args)?;
        // let res = call!(&global_ref, "createEnginesContext", normalize!("(Lcom.pnfsoftware.jeb.core.dao.IDataProvider;Lcom.pnfsoftware.jeb.core.JebClientInformation;)Lcom.pnfsoftware.jeb.core.IEnginesContext;"), &args);
        Ok(Box::new(JebEnginesContext { 0: res }))
    }

    fn closeEnginesContext<'t>(
        &self,
        context: Option<&dyn IEnginesContext>,
    ) -> Result<()> {
        let args: Vec<jni::objects::JValue> = jargs!(context);
        call!(
            self,
            "closeEnginesContext",
            normalize!("(Lcom.pnfsoftware.jeb.core.IEnginesContext;)V"),
            &args
        )?;
        Ok(())
    }
}