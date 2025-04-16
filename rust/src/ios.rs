use jsi::CallInvoker;

#[no_mangle]
pub extern "C" fn Markdown_init(
    rt: *mut jsi::sys::Runtime,
    call_invoker_ptr: *mut std::ffi::c_void, // Now a raw pointer
) {
    if call_invoker_ptr.is_null() {
        eprintln!("CallInvoker pointer is null!");
        return;
    }

    let call_invoker =
        unsafe { &*call_invoker_ptr.cast::<cxx::SharedPtr<jsi::sys::CallInvoker>>() };

    let call_invoker = CallInvoker::new(call_invoker.clone());

    call_invoker.invoke_async(Box::new(move || -> Result<(), anyhow::Error> {
        crate::init(rt);

        Ok(())
    }));
}
