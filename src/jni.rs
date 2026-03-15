#![allow(improper_ctypes_definitions)]
#[allow(non_snake_case)]
use jni::Env;
use jni::objects::{JClass, JString, JValue};
use jni::strings::JNIString;
use jni::sys::{jlong, jobject, jobjectArray, jstring};

use crate::aaml::AAML;

fn throw_java_exception(env: &mut Env<'_>, class: &str, msg: impl ToString) {
    let _ = env.throw_new(JNIString::from(class), JNIString::from(msg.to_string()));
}

fn java_string_to_rust(env: &mut Env<'_>, value: &JString<'_>) -> Result<String, String> {
    value.try_to_string(env).map_err(|e| e.to_string())
}

unsafe fn get_aaml<'a>(ptr: jlong) -> Option<&'a AAML> {
    if ptr == 0 {
        return None;
    }
    Some(unsafe { &*(ptr as *const AAML) })
}

unsafe fn get_aaml_mut<'a>(ptr: jlong) -> Option<&'a mut AAML> {
    if ptr == 0 {
        return None;
    }
    Some(unsafe { &mut *(ptr as *mut AAML) })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_parse<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    content: JString<'local>,
) -> jlong {
    let content = match java_string_to_rust(&mut env, &content) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return 0;
        }
    };

    match AAML::parse(&content) {
        Ok(aaml) => Box::into_raw(Box::new(aaml)) as jlong,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_load<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
) -> jlong {
    let path = match java_string_to_rust(&mut env, &path) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return 0;
        }
    };

    match AAML::load(&path) {
        Ok(aaml) => Box::into_raw(Box::new(aaml)) as jlong,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_merge<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    content: JString<'local>,
) {
    let content = match java_string_to_rust(&mut env, &content) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return;
        }
    };

    let Some(aaml) = (unsafe { get_aaml_mut(ptr) }) else {
        throw_java_exception(
            &mut env,
            "java/lang/IllegalStateException",
            "AamDocument is closed",
        );
        return;
    };

    if let Err(e) = aaml.merge_content(&content) {
        throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_destroy<'local>(
    _env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut AAML)) };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObj<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    let Some(aaml) = (unsafe { get_aaml(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aaml.find_obj(&key) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findKey<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    value: JString<'local>,
) -> jstring {
    let Some(aaml) = (unsafe { get_aaml(ptr) }) else {
        return std::ptr::null_mut();
    };
    let value = match java_string_to_rust(&mut env, &value) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aaml.find_key(&value) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findDeep<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    path: JString<'local>,
) -> jstring {
    let Some(aaml) = (unsafe { get_aaml(ptr) }) else {
        return std::ptr::null_mut();
    };
    let path = match java_string_to_rust(&mut env, &path) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aaml.find_deep(&path) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findList<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobjectArray {
    let Some(aaml) = (unsafe { get_aaml(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let Some(found) = aaml.find_obj(&key) else {
        return std::ptr::null_mut();
    };
    let Some(list) = found.as_list() else {
        return std::ptr::null_mut();
    };

    let class_string = match env.find_class(jni::jni_str!("java/lang/String")) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };
    let initial_str = match env.new_string("") {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };
    let array = match env.new_object_array(list.len() as i32, class_string, initial_str) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    for (i, item) in list.iter().enumerate() {
        let Ok(js) = env.new_string(item) else {
            return std::ptr::null_mut();
        };
        if array.set_element(&mut env, i, js).is_err() {
            return std::ptr::null_mut();
        }
    }

    array.into_raw()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObject<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobject {
    let Some(aaml) = (unsafe { get_aaml(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let Some(found) = aaml.find_obj(&key) else {
        return std::ptr::null_mut();
    };
    let Some(map) = found.as_object() else {
        return std::ptr::null_mut();
    };

    // Use macro for the class name
    let class_hashmap = match env.find_class(jni::jni_str!("java/util/HashMap")) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let hashmap = match env.new_object(&class_hashmap, jni::jni_sig!("()V"), &[]) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    for (k, v) in map {
        let Ok(jk) = env.new_string(k) else {
            return std::ptr::null_mut();
        };
        let Ok(jv) = env.new_string(v) else {
            return std::ptr::null_mut();
        };
        if env
            .call_method(
                &hashmap,
                jni::jni_str!("put"),
                jni::jni_sig!("(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;"),
                &[JValue::Object(&jk.into()), JValue::Object(&jv.into())],
            )
            .is_err()
        {
            return std::ptr::null_mut();
        }
    }

    hashmap.into_raw()
}
