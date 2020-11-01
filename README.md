# JEB API Rust wrappers
This repository uses the jni-rs crate and the Java invocation API to create a JVM and let the JEB-Api run in it. It is not a complete wrapper (yet) but functionality was and is added on demand. Any pull requests are welcome :) 

The examples folder contains an example on how to use the debugger.

## JNI Setup
During compilation, rust tries to find the JavaVM lib via the `JAVA_HOME` environment variable. When running, the `LD_LIBRARY_PATH` should be set, and pointing to the corresponding java installation. For further information, please consult the [jni-rs docs](https://docs.rs/jni/0.18.0/jni/).