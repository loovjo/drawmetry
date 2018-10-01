use super::*;

pub fn default_tools() -> Vec<(MakeButton, PngImage)> {
    vec![
        (cb_set_tool(ToolKind::Point), icons::TOOL_POINT.clone()),
        (cb_set_tool(ToolKind::Circle), icons::TOOL_CIRCLE.clone()),
        (cb_set_tool(ToolKind::Line), icons::TOOL_LINE.clone()),
        (cb_set_tool(ToolKind::Mover), icons::TOOL_MOVER.clone()),
    ]
}

fn cb_set_tool(kind: ToolKind) -> MakeButton {
    MakeButton(Box::new(move || {
        let kind = kind.clone();
        Button {
            function: Box::new(move |state| state.current_tool = kind.clone().into_tool()),
            select: true,
        }
    }))
}

