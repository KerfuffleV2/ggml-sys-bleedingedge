use std::ptr::null_mut;

use ggml_sys_bleedingedge as gg;

#[test]
fn ggml_init_free() {
    unsafe { 
        let params = gg::ggml_init_params {
            mem_size: 1024768,
            mem_buffer: null_mut(),
            no_alloc: false,
        };

        let ctx = gg::ggml_init(params);
        gg::ggml_free(ctx);
    };
}
