use std::panic;

use jni::objects::{JClass, JObject};
use jni::sys::jlong;
use jni::JNIEnv;
use jsi::CallInvoker;

// This function signature allows this function to be called from Java
#[no_mangle]
pub extern "system" fn Java_respond_chat_RespondModule_respond_1markdown_1install<'env>(
    mut env: JNIEnv<'env>,
    _class: JClass<'env>,
    runtime_ptr: jlong,
    call_invoker_holder: JObject<'env>,
) -> () {
    panic::set_hook(Box::new(|info| {
        eprintln!("Rust Panic: {:?}", info);
    }));

    // from https://github.com/facebookincubator/fbjni/blob/8b5aa9eb323184b27b87b5955b242e6e5a342c1a/cxx/fbjni/detail/Hybrid.h
    let hybrid_data: JObject = env
        .get_field(
            call_invoker_holder,
            "mHybridData",
            "Lcom/facebook/jni/HybridData;",
        )
        .expect("can't find hybrid data on CallInvokerHolderImpl")
        .try_into()
        .unwrap();

    let destructor: JObject = env
        .get_field(
            hybrid_data,
            "mDestructor",
            "Lcom/facebook/jni/HybridData$Destructor;",
        )
        .expect("can't find internal destructor on CallInvokerHolderImpl")
        .try_into()
        .unwrap();

    let call_invoker_holder_ptr: jlong = env
        .get_field(destructor, "mNativePointer", "J")
        .expect("can't find native pointer on CallInvokerHolderImpl")
        .try_into()
        .unwrap();

    let call_invoker_holder = call_invoker_holder_ptr as *mut jsi::sys::CallInvokerHolder;
    let call_invoker_holder = unsafe { std::pin::Pin::new_unchecked(&mut *call_invoker_holder) };
    let call_invoker: cxx::SharedPtr<jsi::sys::CallInvoker> = call_invoker_holder.getCallInvoker();
    let call_invoker = CallInvoker::new(call_invoker);

    call_invoker.invoke_async(Box::new(move || -> Result<(), anyhow::Error> {
        let runtime_ptr = runtime_ptr as *mut _;

        crate::init(runtime_ptr);
        Ok(())
    }));
}
