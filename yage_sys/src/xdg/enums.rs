use crate::wl::ObjectId;

pub enum XdgBaseRequest<'req> {
    CreatePositioner { id: &'req ObjectId },
    GetSurface { wl_surface: ObjectId },
    Pong { serial: u32 },
    Destroy,
}

pub enum XdgBaseEvent {
    Ping { serial: u32 },
}

pub enum XdgPositionerRequest {
    SetSize {
        x: i32,
        y: i32,
    },
    SetAnchorRect {
        x: i32,
        y: i32,
        width: i32,
        height: u32,
    },
    SetAnchor {
        anchor: u32,
    },
    SetGravity {
        gravity: u32,
    },
    SetConstraintAdjustment {
        constraint: u32,
    },
    SetOffset {
        x: i32,
        y: i32,
    },
    SetReactive,
    SetParentSize {
        parent_width: i32,
        parent_height: i32,
    },
    Destroy,
}

pub enum XdgPositionerEvent {}

pub enum XdgToplevelRequest<'req> {
    Destroy,
    SetParent {
        obj: Option<&'req ObjectId>,
    },
    SetTitle {
        title: &'req str,
    },
    SetAppId {
        id: &'req str,
    },
    ShowWindowMenu {
        seat: &'req ObjectId,
        serial: u32,
        x: i32,
        y: i32,
    },
    Move {
        seat: &'req ObjectId,
        serial: u32,
    },
    Resize {
        seat: &'req ObjectId,
        serial: u32,
        edges: u32,
    },
    SetMaxSize {
        max_width: i32,
        max_height: i32,
    },
    SetMinSize {
        min_width: i32,
        mid_height: i32,
    },
    SetMaximized,
    UnsetMaximized,
    SetFullscreen {
        output: &'req ObjectId,
    },
    UnsetFullscreen,
    SetMinimized,
}

pub enum XdgToplevelEvent {
    Configure { x: i32, y: i32, array: Vec<u8> },
    Close,
    ConfigureBounds { x: i32, y: i32 },
    Capabilities { array: Vec<u8> },
}

pub enum XdgPopupRequest<'req> {
    Destroy,
    Grab {
        seat: &'req ObjectId,
        serial: u32,
    },
    Repostion {
        positioner: &'req ObjectId,
        serial: u32,
    },
}

pub enum XdgPopupEvents {
    Configure {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Done,
    Repositioned {
        token: u32,
    },
}
