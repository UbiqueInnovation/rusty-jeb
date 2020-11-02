/// This example showcases the use of the JEB Debug API. For the initialization it follows the code
/// found on the JEB github page.
use std::{
    convert::TryInto,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use rusty_jeb::jeb::{
    com::pnfsoftware::jeb::core::{
        units::code::debug::impl_::ValueBoolean, units::code::debug::impl_::ValueInteger,
        units::code::debug::impl_::ValueString, util::IDebuggerThread,
    },
    debug_events::*,
};

use rusty_jeb::jeb::{
    com::pnfsoftware::jeb::core::dao::impl_::DataProvider,
    com::pnfsoftware::jeb::core::properties::impl_::CommonsConfigurationWrapper,
    com::pnfsoftware::jeb::core::units::code::android::dex::DexPoolType,
    com::pnfsoftware::jeb::core::units::JebDexUnit,
    com::pnfsoftware::jeb::core::util::setup_infos::DebuggerSetupInformation,
    com::pnfsoftware::jeb::core::util::DecompilerHelper,
    com::pnfsoftware::jeb::core::{
        dao::impl_::{JEB2FileDatabase, SimpleFSFileStore},
        Artifact, JebCoreService,
    },
    org::apache::commons::configuration2::BaseConfiguration,
    VM,
};

use rusty_jeb::jeb::com::pnfsoftware::jeb::core::units::{IDexUnit, IUnit};

use colored::*;
const PROJECT_NAME: &str = "test";
const LICENSE_KEY: &str = env!(
    "JEB_LICENSE_KEY",
    "Please provide a JEB license key with JEB_LICENSE_KEY='1234'"
);
const PROJECT_FOLDER: &str = ".";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //just attach the current thread to the VM so the subsequent calls are NOPs
    let _ = VM.attach_current_thread_permanently()?;
    let core_service = JebCoreService::getInstance(LICENSE_KEY)?;
    let file_store = SimpleFSFileStore::new(PROJECT_FOLDER)?;
    let projectdb = JEB2FileDatabase::new(PROJECT_FOLDER)?;

    let cfg = BaseConfiguration::new()?;
    let cfg = CommonsConfigurationWrapper::new(cfg)?;

    let data_provider = DataProvider::new(
        None,
        Some(projectdb.as_ref()),
        Some(file_store.as_ref()),
        None,
        None,
        Some(cfg.as_ref()),
    )?;

    //now we are ready to create a CoreContext. The context is the heart of the JEB Api.
    let context = core_service.createEnginesContext(Some(data_provider.as_ref()), None)?;

    //fine, let's start with a new project
    let prj = context.loadProject(PROJECT_NAME)?;
    println!("Created project: {}", prj.getKey()?);

    //we need an artifact
    let artifact = Artifact::new("examples/artifacts/test.apk")?;

    //first let's find the reference to testFunction() to set a breakpoint later on
    let live_artifact = prj.processArtifact(Some(artifact.as_ref()))?;
    //hold a reference to all the units
    let units = live_artifact.getUnits()?;
    //get apk unit
    let apk_unit = units.first().unwrap();
    //hold references to children
    let children = apk_unit.getChildren()?;
    //find dex unit
    let dex_unit: JebDexUnit = children
        .iter()
        .find(|x| x.getFormatType().unwrap_or("".to_string()) == "dex")
        .unwrap()
        .as_ref()
        .try_into()?;
    //get the reference manager
    let ref_manager = dex_unit.getReferenceManager()?;
    //get the decompiler
    let decomp = DecompilerHelper::getDecompiler(Some(&dex_unit))?;

    //hold the first reference to the test function
    let mut ref_to_test_function = None;
    let mut decompiled_function = None;

    //iterate all classes in the dex unit...
    for class in dex_unit.getClasses()? {
        //... and for each class the methods...
        for method in class.getMethods()? {
            //.. to find methods which match our testFunction
            if method.getName(true)?.contains("testFunction") {
                //we want to find method references, referencing the current function
                // jeb uses the dex_pool index to search for references, so we need that
                for reference in
                    ref_manager.getReferences(&DexPoolType::Method, method.getIndex()?)?
                {
                    //we need the internal address to set a breakpoint
                    let ref_func = reference.getInternalAddress()?;
                    ref_to_test_function = Some(ref_func.clone());
                    //if we can compile the method...
                    if decomp.decompileMethod(ref_func.as_str())? {
                        // ...set the code
                        decompiled_function =
                            Some(decomp.getDecompiledMethodText(ref_func.as_str())?);
                    }
                    println!(
                        "Found reference to {} in {}",
                        method.getName(true)?,
                        ref_func
                    );
                    break;
                }
            }
        }
    }

    //hold a reference to all debugger unit identifiers (they have some meta data on the debugger)
    let debuggers = context.getDebuggerUnitIdentifiers()?;
    //find the first one which actually can giive as a targetenumerator, which allows us to retrieve some
    // process informations.
    let debugger = debuggers
        .iter()
        .find(|dbg| dbg.getTargetEnumerator().is_ok())
        .expect("no debugger found");
    // if for some reason the debugger is not enabled, enable it
    if !context.isIdentifierEnabled(Some(debugger.as_ref()))? {
        println!("context not enabled");
        context.setIdentifierEnabled(Some(debugger.as_ref()), true)?;
    }
    //let's hold a reference to the target enumerator
    let target_enumerator = debugger.getTargetEnumerator()?;
    //hold a reference to all machines the debugger can debug on
    let machines = target_enumerator.listMachines()?;
    //since this is a test, we are not really interested in choosing a specific machine, just take the first
    let main_machine = machines
        .first()
        .expect("Target does not have a machine information");
    //we need a list of all processes...
    let processes = main_machine.getProcesses()?;
    //... to find one which matches our criteria
    let mut debug_process = processes.first().unwrap();
    for process in &processes {
        if process.getName()? == "ch.ubique.debugtest1" {
            println!("Found wanted process");
            debug_process = process;
        }
    }
    //there seems to be something wrong with the debuggerhelper. We cannot get it to return a reference to a debugger
    // luckily we have a workaround
    let processor = dex_unit.getUnitProcessor()?;

    //now we can use the unitprocessor to create a debugger unit
    // note though that we need the parent unit to create a debugger!
    let debug_unit = processor.createDebugger("dex_debugger", Some(apk_unit.as_ref()))?;

    //We gathered some intel on the machine before, used it to create a DebuggerSetupInformation
    let info = DebuggerSetupInformation::createWithMachine(
        Some(main_machine.as_ref()),
        Some(debug_process.as_ref()),
    )?;

    //Since we will register a EventHandler on the debugger, we need a way to communicate with this thread
    //so we are going to use channels
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    //lets clone a sender for a custom thread, we will spawn a new thread for terminal input
    let cmd_listener = tx.clone();
    // TL;DR: a java class calls a rust function
    // we use a eventlistener, dynamically registered with the JavaVM. The DebugEventListener is created
    // via the jclass! macro. Have a look at the jni_macros folder on what exactly is generated,
    // or try cargo expand.
    let event_listener = DebugEventListener::new(debug_unit.as_ref(), &dex_unit, tx)?;
    debug_unit.insertListener(0, Some(event_listener.as_ref()))?;

    //so we are ready, make sure Android Studio is shut, and the process up and running (e.g. in the emulator)
    println!("Let's try to attach the debugger");
    while !debug_unit.attach(Some(&info))? {
        println!("Make sure AndroidStudio is shut, and the process running");
        println!("Try again in 5s");
        std::thread::sleep(Duration::from_secs(5));
    }
    //if we found a referencce to the testFunction set a breakpoint
    if let Some(ref_func) = ref_to_test_function {
        debug_unit.setBreakPointWithSymbol(ref_func.as_str(), Some(&dex_unit))?;
    }

    //lets have some references to the currently stopped thread
    let mut current_stopped_thread: Option<Box<dyn IDebuggerThread>> = None;
    let mut current_stopped_thread_id: Option<i64> = None;

    //as promised, spawn a thread listening for terminal input
    let _ = std::thread::spawn(move || {
        let stdin = std::io::stdin();

        loop {
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(_) => cmd_listener.send(input).unwrap(),
                Err(err) => cmd_listener.send(err.to_string()).unwrap(),
            };
        }
    });

    //cool let's start our event loop
    loop {
        //wait for events either from the terminal, or from the DebugEventListener
        let input = rx.recv()?;
        // if we know the thread id for the currently stopped process...
        if let Some(id) = current_stopped_thread_id {
            //... get a reference to it
            current_stopped_thread = Some(debug_unit.getThreadById(id)?);
        } else {
            current_stopped_thread = None;
        }
        //let's sanitize the input
        let input = input.trim().to_string();
        //and split it up into parts
        let mut cmd_args = input.split_whitespace();
        //the first part is always the cmd
        let cmd = cmd_args.next();
        match cmd {
            //let us quit on q
            Some("q") | Some("quit") => {
                break;
            }
            Some("c") | Some("continue") => {
                //if we have a current thread resume the thread
                if let Some(thread) = current_stopped_thread {
                    thread.resume()?;
                    current_stopped_thread_id = None;
                } else {
                    // otherwise try to resume all threads
                    debug_unit.run()?;
                }
            }
            Some("p") | Some("pause") => {
                debug_unit.pause()?;
            }
            //if we have thread we can step over...
            Some("n") | Some("next") => {
                if let Some(thread) = current_stopped_thread.as_deref() {
                    thread.stepOver()?;
                }
            }
            //... or stepinto
            Some("si") | Some("step-in") => {
                if let Some(thread) = current_stopped_thread.as_deref() {
                    thread.stepInto()?;
                }
            }
            //... and eventually stepout. Note though that this only works if we are in a subbranch...
            // one would need to check if this operation is actually allowed
            Some("so") | Some("step-out") => {
                if let Some(thread) = current_stopped_thread.as_deref() {
                    thread.stepOut()?;
                }
            }
            //the set command can be used to set values in the current stack frame
            Some("set") => {
                //we need the name of the register...
                if let Some(what) = cmd_args.next() {
                    //...the type we want to set (e.g. string, integer, boolean)...
                    if let Some(type_string) = cmd_args.next() {
                        //... and collect the rest as the value to be set
                        let val = cmd_args.collect::<Vec<&str>>().join(" ");
                        //so get the current thread...
                        if let Some(thread) = current_stopped_thread.as_deref() {
                            //... get all frames...
                            if let Ok(frames) = &thread.getFrames() {
                                //... and choose the top one (which should be the one used in this context)
                                if let Some(frame) = frames.first() {
                                    // now we need to get all variables available
                                    // WARNING: if the typeHint is set to the wron value manually
                                    //          JEB will crash here, as it just tries to read the memory
                                    //          pointed to by the value (e.g 1 or 0 for booleans, which obviously segfaults)
                                    for var in &frame.getVariables()? {
                                        // if we find a variable matching the one in the set command...
                                        if var.getName()?.as_str() == what {
                                            // ... we check for the type and set the value accordingly
                                            match type_string {
                                                "integer" => {
                                                    let val: i32 = val.parse().unwrap_or(0);
                                                    let val_int = ValueInteger::new(val)?;
                                                    var.setTypedValue(Some(val_int.as_ref()))?;
                                                }
                                                "string" => {
                                                    let val_string =
                                                        ValueString::new(val.to_string())?;
                                                    var.setTypedValue(Some(val_string.as_ref()))?;
                                                }
                                                "boolean" => {
                                                    let val =
                                                        if val == "true" { true } else { false };
                                                    let val_bool = ValueBoolean::new(val)?;
                                                    var.setTypedValue(Some(val_bool.as_ref()))?;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // print the current top level stackframe
            Some("sf") => {
                if let Some(thread) = current_stopped_thread.as_deref() {
                    println!("Local StackFrame\n");
                    // this is analogous to the set command. We get the top level stack frame via the thread
                    if let Ok(frames) = thread.getFrames() {
                        if let Some(frame) = frames.first() {
                            if let Ok(variables) = &frame.getVariables() {
                                for var in variables {
                                    let ty = var.getTypedValue()?;
                                    //try dereference but only if the value is a valid pointer
                                    // since getVariables crashes if the typehint is set to the wrong type
                                    // we check here that we at least only allow values above 0x12000000
                                    // ideally one would get the correct base_address of the Java heap
                                    if &ty.getTypeName()? == "int" {
                                        let ptr: ValueInteger = ty.as_ref().into();
                                        if ptr.getValue()? > 0x12000000 {
                                            var.setTypeHint("string")?;
                                        }
                                    }
                                }
                            }

                            // note that we only get the correct typedvalues if we iterate over all variables again.
                            if let Ok(variables) = &frame.getVariables() {
                                for var in variables {
                                    println!("{}", var.format()?.yellow());
                                    var.setTypeHint("int")?;
                                }
                            }
                        }
                    }
                }
            }
            // our DebugEvenListener will send msg commands to send arbitrary messages
            Some("msg") => {
                let rest: Vec<&str> = cmd_args.collect();
                println!("{}", rest.join(" "));
            }
            // events related to debugging send a dbg event over the channel
            Some("dbg") => match cmd_args.next() {
                //the first dbg message on a breakpoint is the thread id
                Some("tid") => {
                    if let Some(thread_id) = cmd_args.next() {
                        let thread_id = thread_id.parse().unwrap_or(0);
                        current_stopped_thread_id = Some(thread_id);
                    }
                }
                Some("func_exit_bp") => {
                    let rest: Vec<&str> = cmd_args.collect();
                    println!("Function-Exit-Breakpoint, return value: {}");
                }
                // then the bp_hit event will be sent
                Some("bp_hit") => {
                    if let Some(data) = cmd_args.next() {
                        // here we have the address of the breakpoint
                        println!("hit breakpoint {}", data);

                        // and try to get the PC
                        let reg = regex::Regex::new("\\+(?P<pos>(\\d|[AaBbcCdDeEfF])+)h")?;

                        let mut pc = 0;
                        //lets try to convert the hex string to a unsigned 32bit number
                        if let Some(pos) = reg.captures_iter(data).next() {
                            if let Some(position) = pos.name("pos") {
                                let mut pos = position.as_str().to_string();
                                if pos.len() % 2 == 1 {
                                    pos = "0".to_string() + pos.as_str();
                                }
                                let bytes = hex::decode(pos)?;
                                if bytes.len() <= 4 {
                                    let mut byte_arr = [0; 4];
                                    let remainder = 4 - bytes.len();
                                    let mut o = 0;
                                    for i in (remainder..4).rev() {
                                        byte_arr[i] = bytes[o];
                                        o += 1;
                                    }

                                    pc = u32::from_be_bytes(byte_arr);
                                }
                            }
                        }
                        // see if we find the methodd in the codeunit...
                        if let Ok(method) = dex_unit.getMethodByName(data) {
                            //... so we can get all instructions...
                            let instructions = method.getInstructions().unwrap();
                            println!("\n\n{} {{", method.getSignature(false)?);
                            //... and loop over them
                            for instruction in instructions {
                                // get the offset of the instruction...
                                let pos = instruction.getOffset()?;
                                //... andd check if it matches the offset found on the breakpoint
                                if pc as i64 == pos {
                                    // if so, we are currently at this instruction
                                    let current_line = format!(
                                        "==> [{:x}]\t{}\n",
                                        pos,
                                        instruction.format(Some(&dex_unit))?
                                    );
                                    println!("{}", current_line.green());
                                } else {
                                    println!(
                                        "[{:x}]\t{}\n",
                                        pos,
                                        instruction.format(Some(&dex_unit))?
                                    );
                                }
                            }
                            println!("}}\n\n")
                        }
                        // just to get some high level idea of what the current function is doing, also try to get a decompile version of it
                        if decomp.decompileMethod(data)? {
                            let text = decomp.getDecompiledMethodText(data)?;
                            println!("\n\n{}\n\n", text.cyan());
                        } else {
                            println!("Method for address not found {}", data);
                        }
                    }
                }
                _ => {}
            },
            _ => {
                println!("Received unknown command {}", input);
            }
        }
    }
    //we are finished detach the debugger
    debug_unit.detach()?;
    //unload the project...
    context.unloadProject(prj.getKey()?.as_str())?;
    //... and close the engines context
    core_service.closeEnginesContext(Some(context.as_ref()))?;
    //detach the current thread to clean up
    VM.detach_current_thread();
    Ok(())
}
