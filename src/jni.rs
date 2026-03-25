#![allow(improper_ctypes_definitions)]
#[allow(non_snake_case)]
use jni::Env;
use jni::objects::{JClass, JString, JValue};
use jni::strings::JNIString;
use jni::sys::{jlong, jobject, jobjectArray, jstring};

// Use the newly renamed AAM core parser struct, and deprecated AAML alias.
use crate::aam::{AAM, AAML};
use crate::pipeline::formatter::FormatterRules;

fn throw_java_exception(env: &mut Env<'_>, class: &str, msg: impl ToString) {
    let _ = env.throw_new(JNIString::from(class), JNIString::from(msg.to_string()));
}

fn java_string_to_rust(env: &mut Env<'_>, value: &JString<'_>) -> Result<String, String> {
    value.try_to_string(env).map_err(|e| e.to_string())
}

unsafe fn get_aam<'a>(ptr: jlong) -> Option<&'a AAM> {
    if ptr == 0 {
        return None;
    }
    Some(unsafe { &*(ptr as *const AAM) })
}

unsafe fn get_aam_mut<'a>(ptr: jlong) -> Option<&'a mut AAM> {
    if ptr == 0 {
        return None;
    }
    Some(unsafe { &mut *(ptr as *mut AAM) })
}

// -----------------------------------------------------------------------------
// NEW AAM BINDINGS (com.rustgames.aam.AAM)
// -----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_new<'local>(
    mut _env: Env<'local>,
    _class: JClass<'local>,
) -> jlong {
    Box::into_raw(Box::new(AAM::new())) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_parse<'local>(
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

    match AAM::parse(&content) {
        Ok(aam) => Box::into_raw(Box::new(aam)) as jlong,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_load<'local>(
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

    match AAM::load(&path) {
        Ok(aam) => Box::into_raw(Box::new(aam)) as jlong,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_format<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    content: JString<'local>,
) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        throw_java_exception(&mut env, "java/lang/IllegalStateException", "AAM Document is closed");
        return std::ptr::null_mut();
    };

    let content = match java_string_to_rust(&mut env, &content) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return std::ptr::null_mut();
        }
    };

    let rules = FormatterRules::default();
    match aam.format(&content, &rules) {
        Ok(formatted) => match env.new_string(formatted) {
            Ok(js) => js.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_merge<'local>(
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

    let Some(aam) = (unsafe { get_aam_mut(ptr) }) else {
        throw_java_exception(
            &mut env,
            "java/lang/IllegalStateException",
            "AAM Document is closed",
        );
        return;
    };

    if let Err(e) = aam.merge_content(&content) {
        throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_destroy<'local>(
    _env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut AAM)) };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_findObj<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aam.find_obj(&key) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_findKey<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    value: JString<'local>,
) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let value = match java_string_to_rust(&mut env, &value) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aam.find_key(&value) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_findDeep<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    path: JString<'local>,
) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let path = match java_string_to_rust(&mut env, &path) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aam.find_deep(&path) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_findList<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobjectArray {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let Some(found) = aam.find_obj(&key) else {
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
pub extern "system" fn Java_com_rustgames_aam_AAM_findObject<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobject {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let Some(found) = aam.find_obj(&key) else {
        return std::ptr::null_mut();
    };
    let Some(map) = found.as_object() else {
        return std::ptr::null_mut();
    };

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

// -----------------------------------------------------------------------------
// DEPRECATED LEGACY AAML BINDINGS (com.rustgames.aam.AAML)
// Internally routing to the AAM methods to avoid breaking legacy Java code.
// -----------------------------------------------------------------------------

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_new instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_new<'local>(
    env: Env<'local>,
    class: JClass<'local>,
) -> jlong {
    Java_com_rustgames_aam_AAM_new(env, class)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_parse instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_parse<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    content: JString<'local>,
) -> jlong {
    Java_com_rustgames_aam_AAM_parse(env, class, content)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_load instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_load<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    path: JString<'local>,
) -> jlong {
    Java_com_rustgames_aam_AAM_load(env, class, path)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_merge instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_merge<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    content: JString<'local>,
) {
    Java_com_rustgames_aam_AAM_merge(env, class, ptr, content)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_destroy instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_destroy<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
) {
    Java_com_rustgames_aam_AAM_destroy(env, class, ptr)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findObj instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_findObj<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findObj(env, class, ptr, key)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findKey instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_findKey<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    value: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findKey(env, class, ptr, value)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findDeep instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_findDeep<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    path: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findDeep(env, class, ptr, path)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findList instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_findList<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobjectArray {
    Java_com_rustgames_aam_AAM_findList(env, class, ptr, key)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findObject instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAML_findObject<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobject {
    Java_com_rustgames_aam_AAM_findObject(env, class, ptr, key)
}

// -----------------------------------------------------------------------------
// DEPRECATED ORIGINAL BINDINGS (com.aamrs.AamNative)
// Retaining to ensure existing workspace projects linking to the old native don't break.
// -----------------------------------------------------------------------------

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_parse instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_parse<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    content: JString<'local>,
) -> jlong {
    Java_com_rustgames_aam_AAM_parse(env, class, content)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_load instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_load<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    path: JString<'local>,
) -> jlong {
    Java_com_rustgames_aam_AAM_load(env, class, path)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_merge instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_merge<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    content: JString<'local>,
) {
    Java_com_rustgames_aam_AAM_merge(env, class, ptr, content)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_destroy instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_destroy<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
) {
    Java_com_rustgames_aam_AAM_destroy(env, class, ptr)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findObj instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObj<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findObj(env, class, ptr, key)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findKey instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findKey<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    value: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findKey(env, class, ptr, value)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findDeep instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findDeep<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    path: JString<'local>,
) -> jstring {
    Java_com_rustgames_aam_AAM_findDeep(env, class, ptr, path)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findList instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findList<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobjectArray {
    Java_com_rustgames_aam_AAM_findList(env, class, ptr, key)
}

#[deprecated(since = "1.0.0", note = "Use Java_com_rustgames_aam_AAM_findObject instead")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObject<'local>(
    env: Env<'local>,
    class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobject {
    Java_com_rustgames_aam_AAM_findObject(env, class, ptr, key)
}
