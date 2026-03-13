use jni::JNIEnv;
use jni::objects::{JClass, JString, JValue};
use jni::sys::{jlong, jobject, jobjectArray, jstring};

use crate::aaml::AAML;

fn throw_java_exception(env: &mut JNIEnv<'_>, class: &str, msg: impl ToString) {
    let _ = env.throw_new(class, msg.to_string());
}

fn java_string_to_rust(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Result<String, String> {
    env.get_string(value)
        .map(|v| v.into())
        .map_err(|e| e.to_string())
}

unsafe fn get_aaml<'a>(ptr: jlong) -> &'a AAML {
    &*(ptr as *const AAML)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_parse<'local>(
    mut env: JNIEnv<'local>,
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
pub extern "system" fn Java_com_aamrs_AamNative_destroy<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut AAML)) };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObj<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    if ptr == 0 {
        return std::ptr::null_mut();
    }
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let aaml = unsafe { get_aaml(ptr) };
    if let Some(found) = aaml.find_obj(&key) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findDeep<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    path: JString<'local>,
) -> jstring {
    if ptr == 0 {
        return std::ptr::null_mut();
    }
    let path = match java_string_to_rust(&mut env, &path) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let aaml = unsafe { get_aaml(ptr) };
    if let Some(found) = aaml.find_deep(&path) {
        if let Ok(js) = env.new_string(found.as_str()) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findList<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobjectArray {
    if ptr == 0 {
        return std::ptr::null_mut();
    }
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let aaml = unsafe { get_aaml(ptr) };
    let Some(found) = aaml.find_obj(&key) else {
        return std::ptr::null_mut();
    };
    let Some(list) = found.as_list() else {
        return std::ptr::null_mut();
    };

    let class_string = env.find_class("java/lang/String").unwrap();
    let initial_str = env.new_string("").unwrap();
    let array = env
        .new_object_array(list.len() as i32, class_string, initial_str)
        .unwrap();

    for (i, item) in list.iter().enumerate() {
        let js = env.new_string(item).unwrap();
        env.set_object_array_element(&array, i as i32, js).unwrap();
    }

    array.into_raw()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_aamrs_AamNative_findObject<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jobject {
    if ptr == 0 {
        return std::ptr::null_mut();
    }
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let aaml = unsafe { get_aaml(ptr) };
    let Some(found) = aaml.find_obj(&key) else {
        return std::ptr::null_mut();
    };
    let Some(map) = found.as_object() else {
        return std::ptr::null_mut();
    };

    let class_hashmap = env.find_class("java/util/HashMap").unwrap();
    let hashmap = env.new_object(&class_hashmap, "()V", &[]).unwrap();

    for (k, v) in map {
        let jk = env.new_string(k).unwrap();
        let jv = env.new_string(v).unwrap();
        env.call_method(
            &hashmap,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&jk.into()), JValue::Object(&jv.into())],
        )
        .unwrap();
    }

    hashmap.into_raw()
}
