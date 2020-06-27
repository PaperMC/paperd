// This file is part of paperd, the PaperMC server daemon
// Copyright (C) 2019 Kyle Wood (DemonWav)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use jni::objects::JValue::Object;
use jni::objects::{JObject, JString, JThrowable, JValue};
use jni::sys::jobject;
use jni::JNIEnv;
use std::string::String;

pub const JAVA_STRING_TYPE: &'static str = "Ljava/lang/String;";
pub const NPE_CLASS: &'static str = "java/lang/NullPointerException";

pub fn get_path_string(env: &JNIEnv, path: JObject) -> Result<String, ()> {
    let abs_path = match env.call_method(path, "toAbsolutePath", "()Ljava/nio/file/Path;", &[]) {
        Ok(Object(obj)) => obj,
        _ => return Err(()),
    };
    if env.exception_check().unwrap_or(false) {
        return Err(());
    }

    let java_string = match env.call_method(abs_path, "toString", "()Ljava/lang/String;", &[]) {
        Ok(Object(obj)) => obj,
        _ => return Err(()),
    };
    if env.exception_check().unwrap_or(false) {
        return Err(());
    }

    let text = match env.get_string(java_string.into()) {
        Ok(str) => str,
        _ => return Err(()),
    };
    return Ok(text.into());
}

const NATIVE_EXCEPTION_CLASS: &'static str = "com/destroystokyo/paper/daemon/NativeErrorException";
const NATIVE_TIMEOUT_EXCEPTION_CLASS: &'static str =
    "com/destroystokyo/paper/daemon/NativeTimeoutException";
const NATIVE_SOCKET_CLOSED_CLASS: &'static str =
    "com/destroystokyo/paper/daemon/NativeSocketClosedException";

pub fn throw(env: &JNIEnv, message: &str) {
    let _ = env.throw_new(NATIVE_EXCEPTION_CLASS, message);
}

pub fn throw_timeout(env: &JNIEnv) {
    throw_blank(env, NATIVE_TIMEOUT_EXCEPTION_CLASS);
}

pub fn throw_socket_closed(env: &JNIEnv) {
    throw_blank(env, NATIVE_SOCKET_CLOSED_CLASS);
}

fn throw_blank(env: &JNIEnv, class: &str) {
    let obj = env.new_object(class, "()V", &[]);
    if obj.is_ok() {
        let _ = env.throw(JThrowable::from(obj.unwrap()));
    }
}

pub fn throw_with_cause(env: &JNIEnv, message: &str, cause: &JThrowable) {
    let java_string = match env.new_string(message) {
        Ok(string) => string,
        Err(_) => JString::from(jnull!()),
    };

    let ex_obj = match env.new_object(
        NATIVE_EXCEPTION_CLASS,
        "(Ljava/lang/String;Ljava/lang/Throwable;)V",
        &[
            JValue::Object(JObject::from(java_string)),
            JValue::Object(JObject::from(*cause)),
        ],
    ) {
        Ok(obj) => obj,
        Err(_) => {
            // Just attempt to throw an exception with a cause
            throw(env, message);
            return;
        }
    };

    let _ = env.throw(JThrowable::from(ex_obj));
}

pub fn get_class_name(env: &JNIEnv, obj: jobject) -> String {
    return env
        .get_object_class(obj)
        .and_then(|class| env.call_method(class, "getName", "()Ljava/lang/String;", &[]))
        .and_then(|class_name| class_name.l())
        .and_then(|class_name| env.get_string(class_name.into()))
        .map(|str| String::from(str))
        .unwrap_or(String::from("<unknown>"));
}
