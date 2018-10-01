use super::*;
use backend::gwrapper;

pub fn default_toolbar(send: Sender<Button>) -> ToolBar {
    let tools = vec![
        (make_selector(send.clone()), icons::TOOL_SELECTOR.clone()),
        (cb_set_tool(ToolKind::Point), icons::TOOL_POINT.clone()),
        (cb_set_tool(ToolKind::Circle), icons::TOOL_CIRCLE.clone()),
        (cb_set_tool(ToolKind::Line), icons::TOOL_LINE.clone()),
        (cb_set_tool(ToolKind::Mover), icons::TOOL_MOVER.clone()),
    ];

    ToolBar {
        tools: tools,
        send_tool: send,
        selected: Some(0),
    }
}

fn cb_set_tool(kind: ToolKind) -> MakeButton {
    MakeButton(Box::new(move || {
        let kind = kind.clone();
        Button {
            function: Box::new(move |state| state.current_tool = kind.clone().into_tool()),
            select: true,
            subtoolbar: None,
        }
    }))
}

fn make_selector(send: Sender<Button>) -> MakeButton {
    MakeButton(Box::new(move || {
        let tools = vec![
            (make_vis_changer(gwrapper::Visibility::Visible), icons::SELECTED_SHOW.clone()),
            (make_vis_changer(gwrapper::Visibility::Hidden), icons::SELECTED_HIDE.clone()),
        ];

        let subtoolbar = ToolBar {
            tools: tools,
            send_tool: send.clone(),
            selected: None,
        };

        Button {
            function: Box::new(move |_| {}),
            select: true,
            subtoolbar: Some(subtoolbar),
        }
    }))
}

fn make_vis_changer(status: gwrapper::Visibility) -> MakeButton {
    MakeButton(Box::new(move || Button {
        function: Box::new(move |_| println!("Making thing {:?}", status)),
        select: false,
        subtoolbar: None,
    }))
}
