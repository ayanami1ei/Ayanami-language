use inkwell::context::Context;

use crate::lir::LirGenerator;

impl<'a> LirGenerator<'a> {
    pub(crate) fn new() -> LirGenerator<'static> {
        // allocate Context on the heap and leak to obtain a 'static reference
        let ctx_box = Box::new(Context::create());
        let context: &'static Context = Box::leak(ctx_box);
        let module = context.create_module("ayanami_modlue");
        let builder = context.create_builder();
        LirGenerator {
            context,
            module,
            builder,
        }
    }
}
