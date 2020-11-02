// Copyright (c) 2020 Patrick Amrein <amren@ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


#![allow(non_snake_case)]
#![allow(dead_code)]
use std::convert::TryFrom;

use const_format::concatcp;

use jni::{InitArgsBuilder, JNIVersion, JavaVM};
use jni_macros::{define_jclass, ClassFromStr, Instance};

#[macro_use]
pub mod helper_macros;

pub mod file;
pub mod list;
pub mod com;

const JSTRING : &str = "java/lang/String";
const JOBJECT : &str = "java/lang/Object";
const LONG : &str = "J";
const INTEGER : &str = "I";
const BOOL : &str = "Z";


impl<'a, T: Instance + 'a> Instance for Option<T> {
    fn get_obj(&self) -> Result<jni::objects::JObject> {
        if let Some(obj) = self {
            obj.get_obj()
        } else {
            Err("None".into())
        }
    }
}

impl<'a, T: Instance + 'a> Instance for Box<T> {
    fn get_obj(&self) -> Result<jni::objects::JObject> {
        self.as_ref().get_obj()
    }
}

lazy_static! {
    pub static ref VM: JavaVM = {
        let jeb_path =env!("JEB_PATH", "Need a path to jeb to be set");
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option(&format!("-Djava.class.path={}", jeb_path))
            .build()
            .expect("Could not create VMArgs");
        JavaVM::new(jvm_args).unwrap()
    };
}


define_jclass!(
    ()
    package ch.ubique;

    public class NativeEventListener implements com.pnfsoftware.jeb.util.events.IEventListener {
        public native void onEvent(com.pnfsoftware.jeb.util.events.IEvent e);
    }
);

pub mod debug_events {
    use super::*;
    use crate::jeb::com::pnfsoftware::jeb::core::events::ClientNotification;
    use crate::jeb::com::pnfsoftware::jeb::core::units::code::debug::{
        DebuggerEventData, IDebuggerEventData,
    };
    use crate::jeb::com::pnfsoftware::jeb::core::units::IDexUnit;
    use crate::jeb::com::pnfsoftware::jeb::core::util::IDebuggerUnit;
    use crate::jeb::com::pnfsoftware::jeb::util::events::{IEvent, JebEvent};
    use crate::jeb::events::*;
    use std::convert::TryInto;

    define_jclass!(
        (debug_unit : &'a dyn IDebuggerUnit, dex_unit :  &'a dyn IDexUnit<'a>, sender : std::sync::mpsc::Sender<String>)
        package ch.ubique;

        public class DebugEventListener implements com.pnfsoftware.jeb.util.events.IEventListener {
            public native void onEvent(com.pnfsoftware.jeb.util.events.IEvent e);
        }
    );

    impl<'a> IDebugEventListener for DebugEventListener<'a> {
        fn on_event(&self, e: jni::objects::JObject<'_>) {
            let jeb_event = JebEvent(e.into());
            let data: jni::objects::JObject = jeb_event.getType().unwrap();
            match data.into() {
                JebEventType::DbgAttach => {
                    let _ = self.3.send("msg Debugger was attached".to_string());
                }
                JebEventType::DbgDetach => {
                    let _ = self.3.send("msg Debugger was detached".to_string());
                }
                JebEventType::DbgTargetEvent => {
                    let data = jeb_event.getData().unwrap();
                    if let Ok(dbg_event_data) = DebuggerEventData::try_from(data) {
                        match IDebuggerEventData::getType(&dbg_event_data).unwrap() {
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Breakpoint => {
                                let _ = self.3.send(format!("dbg tid {}", dbg_event_data.getThreadId().unwrap_or(0)));
                                let _ = self.3.send(format!("dbg bp_hit {}",dbg_event_data.getAddress().unwrap_or_else(|_|"".to_string()) ));
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::BreakpointFunctionExit => {
                                let _ = self.3.send(format!("dbg tid {}", dbg_event_data.getThreadId().unwrap_or(0)));
                                let _ = self.3.send(format!("dbg bp_hit {}",dbg_event_data.getAddress().unwrap_or_else(|_|"".to_string()) ));
                                if let Ok(ret_val) =dbg_event_data.getReturnValue() {
                                    let _ = self.3.send(format!("dbg func_exit_bp {}", ret_val.format().unwrap_or("".to_string())));
                                } 
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::CodeLoad => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::CodeUnload => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Exception => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::FunctionEntry => {
                                let _ = self.3.send("msg FunctionEntry".to_string());
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::FunctionExit => {
                                let _ = self.3.send("msg FunctionExit".to_string());
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Output => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Signal => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Suspended => {}
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::ThreadStart => {
                                let _ = self.3.send("msg ThreadStart".to_string());
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::ThreadStop => {
                                let _ = self.3.send("msg ThreadStop".to_string());
                            }
                            com::pnfsoftware::jeb::core::units::code::debug::DebuggerEventType::Unknown => {}
                        }
                    } else {
                        let _ = self
                            .3
                            .send("msg data was not a DebuggerEventData".to_string());
                    }
                }
                JebEventType::DbgBreakpointSet => {
                    let _ = self.3.send("msg Set breakpoint".to_string());
                }
                JebEventType::Notification | JebEventType::DbgClientNotification => {
                    let data = jeb_event.getData().unwrap();
                    let client_notification: ClientNotification = data.try_into().unwrap();

                    let _ = self.3.send(format!(
                        "msg [{:?}] {}",
                        client_notification.getLevel().unwrap(),
                        client_notification.getMessage().unwrap()
                    ));
                }
                _ => {
                    let event: JebEventType = data.into();
                    let _ = self
                        .3
                        .send(format!("msg Non debugging related event {:?}", event));
                }
            }
        }
    }
}

mod event_listener {
    use super::*;
    use crate::jeb::events::JebEventType;
    use crate::jeb::com::pnfsoftware::jeb::util::events::{IEvent, JebEvent};

    impl<'a> INativeEventListener for NativeEventListener<'a> {
        fn on_event(&self, e: jni::objects::JObject) {
            let jeb_event = JebEvent(e.into());
            let data: jni::objects::JObject = jeb_event.getType().unwrap();
            match data.into() {
                JebEventType::UnitProcessed => println!("UnitProcessed Event"),
                JebEventType::UnitCreated => {
                    println!("Unit created Event");
                }
                _ => {}
            }
        }
    }
}

mod events {
    static PACKAGE_NAME: &str = "com.pnfsoftware.jeb.util.events";

    use super::*;

    #[derive(Debug)]
    pub enum JebEventType {
        ArtifactProcessed,
        CoreError,
        DbgAttach,
        DbgClientNotification,
        DbgDetach,
        DbgPause,
        DbgRun,
        DbgTargetEvent,
        DbgBreakpointSet,
        DecompClientNotification,
        DecompSrcUnitResetEvent,
        UnitCreated,
        UnitDestroyed,
        UnitProcessed,
        Notification,
        Unknown,
    }

    impl JebEventType {
        fn isArtifactEvent(ty: JebEventType) -> bool {
            matches!(ty, JebEventType::ArtifactProcessed)
        }
        fn isDebuggerEvent(ty: JebEventType) -> bool {
            matches!(
                ty,
                JebEventType::DbgAttach
                    | JebEventType::DbgDetach
                    | JebEventType::DbgPause
                    | JebEventType::DbgRun
                    | JebEventType::DbgTargetEvent
                    | JebEventType::DbgBreakpointSet
                    | JebEventType::DbgClientNotification
            )
        }
        fn isDecompilerEvent(ty: JebEventType) -> bool {
            matches!(
                ty,
                JebEventType::DecompClientNotification | JebEventType::DecompSrcUnitResetEvent
            )
        }
        fn isUnitEvent(ty: JebEventType) -> bool {
            matches!(
                ty,
                JebEventType::UnitCreated
                    | JebEventType::UnitProcessed
                    | JebEventType::UnitDestroyed
            )
        }
    }

    impl<'a> From<jni::objects::JObject<'a>> for JebEventType {
        fn from(data: jni::objects::JObject<'a>) -> Self {
            let env = get_vm_unwrap!();
            if let Ok(res) = env.call_method(data, "toString", "()Ljava/lang/String;", &[]) {
                let string: String = env
                    .get_string(res.l().unwrap().into())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                match string.as_str() {
                    "ArtifactProcessed" => JebEventType::ArtifactProcessed,
                    "CoreError" => JebEventType::CoreError,
                    "DbgAttach" => JebEventType::DbgAttach,
                    "DbgClientNotification" => JebEventType::DbgClientNotification,
                    "DbgDetach" => JebEventType::DbgDetach,
                    "DbgPause" => JebEventType::DbgPause,
                    "DbgRun" => JebEventType::DbgRun,
                    "DbgTargetEvent" => JebEventType::DbgTargetEvent,
                    "DbgBreakpointSet" => JebEventType::DbgBreakpointSet,
                    "DecompClientNotification" => JebEventType::DecompClientNotification,
                    "DecompSrcUnitResetEvent" => JebEventType::DecompSrcUnitResetEvent,
                    "UnitCreated" => JebEventType::UnitCreated,
                    "UnitDestroyed" => JebEventType::UnitDestroyed,
                    "UnitProcessed" => JebEventType::UnitProcessed,
                    "Notification" => JebEventType::Notification,
                    _ => JebEventType::Unknown,
                }
            } else {
                JebEventType::Unknown
            }
        }
    }

    
    jclass!(JebDebugEvent, JebDebugEvent_);
    jclass!(JebDebugEventData, JebDebugEventData_);
    use crate::jeb::com::pnfsoftware::jeb::util::events::{IEvent};
   
    pub trait IDebuggerEventData<'a>: Instance {}
    pub trait IDebugEvent<'a>: IEvent<'a> {
        fn getData(&self) -> Result<Box<dyn IDebuggerEventData + '_>>;
    }

    impl<'a> IDebuggerEventData<'a> for JebDebugEventData<'a> {}

    impl<'a> IDebugEvent<'a> for JebDebugEvent<'a> {
        propagate_interface! {
            [IEvent]
            [IDebuggerEventData]
            [JebDebugEventData]
            fn getData()
        }
    }
}

type Result<'a, T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub trait Instance {
    fn get_obj(&self) -> Result<jni::objects::JObject>;
}

pub mod org {
    pub mod apache {
        pub mod commons {
            pub mod configuration2 {
                static PACKAGE_NAME: &str = "org/apache/commons/configuration2";
                use crate::jeb::*;

                jclass! {BaseConfiguration, BaseConfiguration_}
                impl<'a> BaseConfiguration<'a> {
                    pub fn new() -> Result<'a, BaseConfiguration<'a>> {
                        let env = get_vm!();
                        let res = env.new_object(BaseConfiguration_, "()V", &[])?;
                        Ok(BaseConfiguration(res.into()))
                    }

                    pub fn set_property(
                        &self,
                        key: String,
                        value: jni::objects::JObject,
                    ) -> Result<()> {
                        let env = VM.attach_current_thread_permanently()?;
                        let args: Vec<jni::objects::JValue> =
                            vec![env.new_string(key)?.into(), value.into()];
                        if let jni::objects::JValue::Object(obj) = self.0 {
                            env.call_method(
                                obj,
                                "setProperty",
                                "(Ljava/lang/String;Ljava/lang/Object;)V",
                                &args,
                            )?;

                            Ok(())
                        } else {
                            Err("Instance is not object".into())
                        }
                    }
                }
            }
        }
    }
}
