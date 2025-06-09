use super::{bindings, ArgKind, Interface, Message};

pub static WL_DISPLAY_INTERFACE: Interface = Interface::new("wl_display", 1)
    .requests(&[
        Message::with_sig("sync", &[ArgKind::New]).child(&WL_CALLBACK_INTERFACE),
        Message::with_sig("get_registry", &[ArgKind::New]).child(&WL_REGISTRY_INTERFACE),
    ])
    .events(&[
        Message::with_sig("error", &[ArgKind::New]).child(&WL_REGISTRY_INTERFACE),
        Message::with_sig("delete_id", &[ArgKind::Uint]),
    ])
    .interface(unsafe { &bindings::wl_display_interface });

pub static WL_REGISTRY_INTERFACE: Interface = Interface::new("wl_registry", 1)
    .requests(&[Message::with_sig("bind", &[ArgKind::Uint, ArgKind::New])])
    .events(&[
        Message::with_sig(
            "global",
            &[ArgKind::Uint, ArgKind::Str(false), ArgKind::Uint],
        ),
        Message::with_sig("global_remove", &[ArgKind::Uint]),
    ])
    .interface(unsafe { &bindings::wl_registry_interface });

pub static WL_CALLBACK_INTERFACE: Interface = Interface::new("wl_callback", 1)
    .events(&[Message::with_sig("done", &[ArgKind::Uint]).destructor(true)])
    .interface(unsafe { &bindings::wl_callback_interface });

pub static WL_SURFACE_IFACE: Interface = Interface::new("wl_surface", 6)
    .events(&[
        Message::DESTROY,
        Message::with_sig(
            "attach",
            &[ArgKind::Object(true), ArgKind::Int, ArgKind::Int],
        )
        .ifaces(&[&WL_BUFFER_IFACE]),
        Message::with_sig(
            "damage",
            &[ArgKind::Int, ArgKind::Int, ArgKind::Int, ArgKind::Int],
        ),
        Message::with_sig("frame", &[ArgKind::New]).child(&WL_CALLBACK_INTERFACE),
        Message::with_sig("set_opaque_region", &[ArgKind::Object(true)])
            .ifaces(&[&WL_REGION_IFACE]),
        Message::with_sig("set_input_region", &[ArgKind::Object(true)]).ifaces(&[&WL_REGION_IFACE]),
        Message::with_sig("commit", &[]),
        Message::with_sig("set_buffer_transform", &[ArgKind::Int]).since(2),
        Message::with_sig("set_buffer_scale", &[ArgKind::Int]).since(3),
        Message::with_sig("damage_buffer", &[ArgKind::Int, ArgKind::Int]).since(4),
        Message::with_sig("offset", &[ArgKind::Int, ArgKind::Int]).since(5),
    ])
    .events(&[
        Message::with_sig("enter", &[ArgKind::Object(false)]).ifaces(&[&WL_OUTPUT_IFACE]),
        Message::with_sig("leave", &[ArgKind::Object(false)]).ifaces(&[&WL_OUTPUT_IFACE]),
        Message::with_sig("preferred_buffer_scale", &[ArgKind::Int]).since(6),
        Message::with_sig("preferred_buffer_transform", &[ArgKind::Uint]).since(6),
    ])
    .interface(unsafe { &bindings::wl_surface_interface });

pub static WL_OUTPUT_IFACE: Interface = Interface::new("wl_output", 4)
    .requests(&[Message::new("release").destructor(true).since(3)])
    .events(&[
        Message::with_sig(
            "geometry",
            &[
                ArgKind::Int,
                ArgKind::Int,
                ArgKind::Int,
                ArgKind::Int,
                ArgKind::Int,
                ArgKind::Str(false),
                ArgKind::Str(false),
                ArgKind::Int,
            ],
        ),
        Message::with_sig(
            "mode",
            &[ArgKind::Uint, ArgKind::Int, ArgKind::Int, ArgKind::Int],
        ),
        Message::new("done").since(2),
        Message::with_sig("scale", &[ArgKind::Int]).since(2),
        Message::with_sig("name", &[ArgKind::Str(false)]).since(4),
        Message::with_sig("description", &[ArgKind::Str(false)]).since(4),
    ])
    .interface(unsafe { &bindings::wl_output_interface });

pub static WL_REGION_IFACE: Interface = Interface::new("wl_region", 4)
    .requests(&[
        Message::DESTROY,
        Message::with_sig("add", &[ArgKind::Int; 4]),
        Message::with_sig("subtract", &[ArgKind::Int; 4]),
    ])
    .interface(unsafe { &bindings::wl_region_interface });

pub static WL_BUFFER_IFACE: Interface = Interface::new("wl_buffer", 1)
    .requests(&[Message::DESTROY])
    .events(&[Message::new("release")])
    .interface(unsafe { &bindings::wl_buffer_interface });

pub static WL_SEAT_IFACE: Interface = Interface::new("wl_seat", 10)
    .requests(&[
        Message::with_sig("get_pointer", &[ArgKind::New]).child(&WL_POINTER_IFACE),
        Message::with_sig("get_keyboard", &[ArgKind::New]).child(&WL_KEYBOARD_IFACE),
        Message::with_sig("get_touch", &[ArgKind::New]).child(&WL_TOUCH_IFACE),
        Message::with_sig("release", &[]).destructor(true).since(5),
    ])
    .events(&[
        Message::with_sig("capabilites", &[ArgKind::Uint]),
        Message::with_sig("name", &[ArgKind::Str(false)]).since(2),
    ])
    .interface(unsafe { &bindings::wl_seat_interface });

pub static WL_POINTER_IFACE: Interface = Interface::new("wl_pointer", 10)
    .requests(&[
        Message::with_sig(
            "set_cursor",
            &[
                ArgKind::Uint,
                ArgKind::Object(true),
                ArgKind::Int,
                ArgKind::Int,
            ],
        )
        .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("release", &[]).since(3).destructor(true),
    ])
    .events(&[
        Message::with_sig(
            "enter",
            &[
                ArgKind::Uint,
                ArgKind::Object(false),
                ArgKind::Int,
                ArgKind::Int,
            ],
        )
        .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("leave", &[ArgKind::Uint, ArgKind::Object(false)])
            .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("motion", &[ArgKind::Uint, ArgKind::Int, ArgKind::Int]),
        Message::with_sig(
            "button",
            &[ArgKind::Uint, ArgKind::Uint, ArgKind::Uint, ArgKind::Uint],
        ),
        Message::with_sig("axis", &[ArgKind::Uint, ArgKind::Uint, ArgKind::Fixed]),
        Message::with_sig("frame", &[]).since(5),
        Message::with_sig("axis_source", &[ArgKind::Uint]).since(5),
        Message::with_sig("axis_stop", &[ArgKind::Uint, ArgKind::Uint]).since(5),
        Message::with_sig("axis_discrete", &[ArgKind::Uint, ArgKind::Int]).since(5),
        Message::with_sig("axis_value120", &[ArgKind::Uint, ArgKind::Int]).since(8),
        Message::with_sig("axis_relative_direction", &[ArgKind::Uint, ArgKind::Uint]).since(9),
    ])
    .interface(unsafe { &bindings::wl_pointer_interface });

pub static WL_KEYBOARD_IFACE: Interface = Interface::new("wl_keyboard", 10)
    .requests(&[Message::new("release").destructor(true).since(3)])
    .events(&[
        Message::with_sig("keymap", &[ArgKind::Uint, ArgKind::Fd, ArgKind::Uint]),
        Message::with_sig(
            "enter",
            &[ArgKind::Uint, ArgKind::Object(false), ArgKind::Array],
        )
        .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("leave", &[ArgKind::Uint, ArgKind::Object(false)])
            .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("key", &[ArgKind::Uint; 4]),
        Message::with_sig("modifiers", &[ArgKind::Uint; 5]),
        Message::with_sig("repeat_info", &[ArgKind::Int, ArgKind::Int]).since(4),
    ])
    .interface(unsafe { &bindings::wl_keyboard_interface });

pub static WL_TOUCH_IFACE: Interface = Interface::new("wl_touch", 10)
    .requests(&[Message::new("release").destructor(true).since(3)])
    .events(&[
        Message::with_sig(
            "down",
            &[
                ArgKind::Uint,
                ArgKind::Uint,
                ArgKind::Object(false),
                ArgKind::Int,
                ArgKind::Fixed,
                ArgKind::Fixed,
            ],
        )
        .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("up", &[ArgKind::Uint, ArgKind::Uint, ArgKind::Int]),
        Message::with_sig(
            "motion",
            &[ArgKind::Uint, ArgKind::Int, ArgKind::Fixed, ArgKind::Fixed],
        ),
        Message::new("frame"),
        Message::new("cancel"),
        Message::with_sig("shape", &[ArgKind::Int, ArgKind::Fixed, ArgKind::Fixed]).since(6),
        Message::with_sig("orientation", &[ArgKind::Int, ArgKind::Fixed]).since(6),
    ])
    .interface(unsafe { &bindings::wl_touch_interface });

#[cfg(test)]
mod tests {
    use std::{ffi::CStr, fmt, ptr::NonNull};

    use super::*;

    struct DebugHelper(NonNull<bindings::wl_interface>);

    #[repr(transparent)]
    struct MessageDebugHelper(bindings::wl_message);

    impl fmt::Debug for MessageDebugHelper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let bindings::wl_message {
                name, signature, ..
            } = self.0;

            let name = unsafe { CStr::from_ptr(name) };

            let sig = unsafe { CStr::from_ptr(signature) };

            f.debug_struct("wl_message")
                .field("name", &name)
                .field("sig", &sig)
                .finish_non_exhaustive()
        }
    }

    impl fmt::Debug for DebugHelper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let iface = unsafe { self.0.as_ref() };
            let name = unsafe { CStr::from_ptr(iface.name) };
            let method_slice = unsafe {
                core::slice::from_raw_parts(
                    iface.methods as *const MessageDebugHelper,
                    iface.method_count as _,
                )
            };
            let event_slice = unsafe {
                core::slice::from_raw_parts(
                    iface.methods as *const MessageDebugHelper,
                    iface.event_count as _,
                )
            };

            f.debug_struct("Interface")
                .field("name", &name)
                .field("version", &iface.version)
                .field("num_methods", &iface.method_count)
                .field("num_events", &iface.event_count)
                .field("methods", &method_slice)
                .field("events", &event_slice)
                .finish_non_exhaustive()
        }
    }

    #[test]
    fn test_iface() {
        let dbg = unsafe {
            DebugHelper(NonNull::new_unchecked(
                (&raw const bindings::wl_display_interface) as *mut _,
            ))
        };

        println!("{dbg:#?}");
    }
}
