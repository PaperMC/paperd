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

macro_rules! jnull {
    () => {
        jni::objects::JObject::null().into_inner()
    };
}

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
                let error_msg = format!(stringify!(Failed to get $name from {}), class_name.as_str());
                throw(&$env, error_msg.as_str());
                return;
            }
        };
    };
}

macro_rules! handle_syscall {
    ($env:ident, $call:expr, $return:expr) => {
        match $call {
            Ok(v) => v,
            Err(e) => {
                let error_msg = format!("Error attempting system call: {}", e);
                throw(&$env, error_msg.as_str());
                return $return;
            }
        }
    };
}
