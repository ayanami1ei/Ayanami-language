#[macro_export]
macro_rules! make_fn {
    ($ctx:expr, $name:ident) => {{
        let obj_ptr = $ctx.context.i8_type().ptr_type(AddressSpace::default());
        let f = $ctx.module.add_function(
            stringify!($name),
            obj_ptr.fn_type(&[obj_ptr.into(), obj_ptr.into()], false),
            None,
        );
        $ctx.runtime_fn.insert(stringify!($name), f);
    }};
}

#[macro_export]
macro_rules! bin_operator_fn {
    ($ctx:expr, $($name:ident),* $(,)?) => {
        $(
            make_fn!($ctx, $name);
        )*
    };
}

#[macro_export]
macro_rules! call_bin_operator_fn {
    ($ctx:expr, $name:ident, $left_obj:ident, $right_obj:ident, $call:ident)=>{{
        let f = *$ctx.runtime_fn.get(stringify!($name)).expect("runtime not init");
        $call = $ctx
            .builder
            .build_call(
                f,
                &[$left_obj.into(), $right_obj.into()],
                stringify!($name),
            )
            .unwrap();
    }
}}
