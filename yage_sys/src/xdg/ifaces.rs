use crate::wl::{
    interfaces::{self, WL_SURFACE_IFACE},
    ArgKind, Interface, Message,
};

pub static XDG_WM_IFACE: Interface = Interface::new("xdg_wm_base", 7)
    .requests(&[
        Message::DESTROY,
        Message::with_sig("create_positioner", &[ArgKind::New]).child(&XDG_POSITIONER_IFACE),
        Message::with_sig("get_xdg_surface", &[ArgKind::New, ArgKind::Object(false)])
            .child(&XDG_SURFACE_IFACE)
            .ifaces(&[&WL_SURFACE_IFACE]),
        Message::with_sig("pong", &[ArgKind::Uint]),
    ])
    .events(&[Message::with_sig("ping", &[ArgKind::Uint])]);

pub static XDG_POSITIONER_IFACE: Interface = Interface::new("xdg_positioner", 7).requests(&[
    Message::DESTROY,
    Message::with_sig("set_size", &[ArgKind::Int, ArgKind::Int]),
    Message::with_sig(
        "set_anchor_rect",
        &[ArgKind::Int, ArgKind::Int, ArgKind::Int, ArgKind::Int],
    ),
    Message::with_sig("set_anchor", &[ArgKind::Uint]),
    Message::with_sig("set_gravity", &[ArgKind::Uint]),
    Message::with_sig("set_constraint_adjustment", &[ArgKind::Uint]),
    Message::with_sig("set_offset", &[ArgKind::Int, ArgKind::Int]),
    Message::with_sig("set_reactive", &[]).since(3),
    Message::with_sig("set_parent_size", &[ArgKind::Int, ArgKind::Int]).since(3),
]);

pub static XDG_SURFACE_IFACE: Interface = Interface::new("xdg_surface", 7)
    .requests(&[
        Message::DESTROY,
        Message::with_sig("set_toplevel", &[ArgKind::New]).child(&XDG_TOPLEVEL_IFACE),
        Message::with_sig(
            "get_popup",
            &[ArgKind::New, ArgKind::Object(true), ArgKind::Object(false)],
        )
        .child(&XDG_POPUP_IFACE)
        .ifaces(&[&XDG_SURFACE_IFACE, &XDG_POSITIONER_IFACE]),
        Message::with_sig(
            "set_window_geometry",
            &[ArgKind::Int, ArgKind::Int, ArgKind::Int, ArgKind::Int],
        ),
        Message::with_sig("ack_configure", &[ArgKind::Uint]),
    ])
    .events(&[Message::with_sig("configure", &[ArgKind::Uint])]);

pub static XDG_TOPLEVEL_IFACE: Interface = Interface::new("xdg_toplevel", 7)
    .requests(&[
        Message::DESTROY,
        Message::with_sig("set_parent", &[ArgKind::Object(true)]).ifaces(&[&XDG_TOPLEVEL_IFACE]),
        Message::with_sig("set_title", &[ArgKind::Str(false)]),
        Message::with_sig("set_app_id", &[ArgKind::Str(false)]),
        Message::with_sig(
            "show_window_menu",
            &[
                ArgKind::Object(false),
                ArgKind::Uint,
                ArgKind::Int,
                ArgKind::Int,
            ],
        )
        .ifaces(&[&interfaces::WL_SEAT_IFACE]),
        Message::with_sig("move", &[ArgKind::Object(false), ArgKind::Uint])
            .ifaces(&[&interfaces::WL_SEAT_IFACE]),
        Message::with_sig(
            "resize",
            &[ArgKind::Object(false), ArgKind::Uint, ArgKind::Uint],
        )
        .ifaces(&[&interfaces::WL_SEAT_IFACE]),
        Message::with_sig("set_max_size", &[ArgKind::Int, ArgKind::Int]),
        Message::with_sig("set_min_size", &[ArgKind::Int, ArgKind::Int]),
        Message::with_sig("set_maximized", &[]),
        Message::with_sig("unset_maximized", &[]),
        Message::with_sig("set_fullscreen", &[ArgKind::Object(true)])
            .ifaces(&[&interfaces::WL_OUTPUT_IFACE]),
        Message::with_sig("unset_fullscreen", &[]),
        Message::with_sig("set_minimized", &[]),
    ])
    .events(&[
        Message::with_sig("configure", &[ArgKind::Int, ArgKind::Int, ArgKind::Array]),
        Message::with_sig("close", &[]),
        Message::with_sig("configure_bounds", &[ArgKind::Int, ArgKind::Int]).since(4),
        Message::with_sig("wm_capabilities", &[ArgKind::Array]).since(5),
    ]);

pub static XDG_POPUP_IFACE: Interface = Interface::new("xdg_popup", 7)
    .requests(&[
        Message::DESTROY,
        Message::with_sig("grab", &[ArgKind::Object(false), ArgKind::Uint])
            .ifaces(&[&interfaces::WL_SEAT_IFACE]),
        Message::with_sig("reposition", &[ArgKind::Object(false), ArgKind::Uint])
            .since(3)
            .ifaces(&[&XDG_POSITIONER_IFACE]),
    ])
    .events(&[
        Message::with_sig(
            "configure",
            &[ArgKind::Int, ArgKind::Int, ArgKind::Int, ArgKind::Int],
        ),
        Message::with_sig("popup_done", &[]),
        Message::with_sig("repositioned", &[ArgKind::Uint]).since(3),
    ]);
