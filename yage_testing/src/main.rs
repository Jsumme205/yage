use std::ffi::CString;
use std::ptr::NonNull;

use yage_core::allocator_api::Allocator;
use yage_sys::{
    shader::{FragmentShader, ShaderLoader, VertexShader},
    window::RawWindowParams,
};

/// extremely unsafe, only used to test conditions
struct StdAlloc;

unsafe impl Allocator for StdAlloc {
    type Error = ();

    fn allocate(
        &mut self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<u8>, Self::Error> {
        unsafe { Ok(NonNull::new_unchecked(std::alloc::alloc(layout))) }
    }

    fn deallocate(&mut self, ptr: *mut u8, layout: std::alloc::Layout) -> Result<(), Self::Error> {
        unsafe { std::alloc::dealloc(ptr, layout) };
        Ok(())
    }

    fn reallocate(
        &mut self,
        old_ptr: *mut u8,
        old_layout: std::alloc::Layout,
        new_layout: std::alloc::Layout,
    ) -> Result<NonNull<u8>, Self::Error> {
        Err(())
    }
}

fn main() {
    yage_sys::glfw_init().unwrap();
    yage_sys::register_error_callback(yage_sys::DefaultErrorCallback);

    /*
    let mut vertex_shader: ShaderLoader<6, VertexShader> =
        ShaderLoader::load("vertex-shader.vs").unwrap();
    vertex_shader
        .push_attr_line([0.5, -0.5, 0.0, 1.0, 0.0, 0.0])
        .push_attr_line([-0.5, -0.5, 0.0, 0.0, 1.0, 0.0])
        .push_attr_line([0.0, 0.5, 0.0, 0.0, 0.0, 1.0]);
    let mut v_compiled = unsafe { vertex_shader.compile() };

    let mut frag_shader: ShaderLoader<6, FragmentShader> =
        ShaderLoader::load("frag-shader.fs").unwrap();
    frag_shader
        .push_attr_line([0.5, -0.5, 0.0, 1.0, 0.0, 0.0])
        .push_attr_line([-0.5, -0.5, 0.0, 0.0, 1.0, 0.0])
        .push_attr_line([0.0, 0.5, 0.0, 0.0, 0.0, 1.0]);
    let mut f_compiled = unsafe { frag_shader.compile() };

    */

    let mut window = yage_sys::window::RawWindow::create(RawWindowParams {
        width: 200,
        height: 200,
        name: Some(CString::new("test").unwrap()),
        key_handler: None,
    })
    .unwrap();

    //v_compiled.use_shader(&window);
    //f_compiled.use_shader(&window);

    //let _ = unsafe { window.main_loop(|_| Ok(())) };

    //println!("{:?}", handle.join());

    println!("Hello, world!");
}
