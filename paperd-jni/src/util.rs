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

use alloc::string::String;
use jni::objects::JObject;
use jni::objects::JValue::Object;
use jni::JNIEnv;

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

pub fn throw(env: &JNIEnv, message: &str) {
    let _ = env.throw((
        "com/destroystokyo/paper/daemon/NativeErrorException",
        message,
    ));
}

pub fn get_class_name(env: &JNIEnv, obj: JObject) -> String {
    return env
        .get_object_class(obj)
        .and_then(|class| env.call_method(class.into(), "getName", "()Ljava/lang/String;", &[]))
        .and_then(|class_name| class_name.l())
        .and_then(|class_name| env.get_string(class_name.into()))
        .map(|str| String::from(str))
        .unwrap_or(String::from("<unknown>"));
}

// I have no idea if there's a better way of doing this...I really hope there is
macro_rules! get_field {
    ($env:ident, $obj:ident, $name:expr, Object($ty:expr)) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Object, $ty))
    };
    ($env:ident, $obj:ident, $name:expr, Byte) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Byte, "B"))
    };
    ($env:ident, $obj:ident, $name:expr, Char) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Char, "C"))
    };
    ($env:ident, $obj:ident, $name:expr, Short) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Short, "S"))
    };
    ($env:ident, $obj:ident, $name:expr, Int) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Int, "I"))
    };
    ($env:ident, $obj:ident, $name:expr, Long) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Long, "J"))
    };
    ($env:ident, $obj:ident, $name:expr, Bool) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Bool, "Z"))
    };
    ($env:ident, $obj:ident, $name:expr, Float) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Float, "F"))
    };
    ($env:ident, $obj:ident, $name:expr, Double) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Double, "D"))
    };
    ($env:ident, $obj:ident, $name:expr, Void) => {
        get_field!($env, $obj, $name, (jni::objects::JValue::Void, "V"))
    };
    ($env:ident, $obj:ident, $name:expr, ($ret:path, $ty:expr)) => {
        match $env.get_field($obj, $name, $ty) {
            Ok($ret(t)) => t,
            _ => {
                let class_name = crate::util::get_class_name(&$env, $obj);
                let mut err_str = alloc::string::String::from(stringify!(Failed to get $name from ));
                err_str.push_str(class_name.as_str());
                throw(&$env, err_str.as_str());
                return;
            }
        };
    };
}

macro_rules! check_err {
    ($env:ident, $val:expr) => {{
        let ret = $val;
        if ret == -1 {
            let msg = Errno::last().desc();
            throw(&$env, msg);
            return;
        }
        ret
    }};

    ($env:ident, $val:expr, $ret_val:expr) => {{
        let ret = $val;
        if ret == -1 {
            let msg = Errno::last().desc();
            throw(&$env, msg);
            return $ret_val;
        }
        ret
    }};
}
