#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImeStage {
    Nothing,
    SelectAllDown,
    SelectAllUp,
    BackSpaceDown,
    BackSpaceUp,
    PutText,
    EscapeDown,
    EscapeUp,
}

impl ImeStage {
    pub(crate) const START: ImeStage = ImeStage::SelectAllDown;
    pub(crate) const CANCEL: ImeStage = ImeStage::EscapeDown;
    pub(crate) fn next(self) -> Self {
        use ImeStage::*;
        match self {
            Nothing => Nothing,
            SelectAllDown => SelectAllUp,
            SelectAllUp => BackSpaceDown,
            BackSpaceDown => BackSpaceUp,
            BackSpaceUp => PutText,
            PutText => EscapeDown,
            EscapeDown => EscapeUp,
            EscapeUp => Nothing,
        }
    }
    pub(crate) fn advance(&mut self) {
        *self = self.next();
    }
    pub(crate) fn add_event(self, events: &mut Vec<egui::Event>) -> bool {
        use ImeStage::*;
        match self {
            Nothing => false,
            SelectAllDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: true,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            SelectAllUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: false,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            BackSpaceDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            BackSpaceUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            PutText => true,
            EscapeDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            EscapeUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
        }
    }
}

pub(crate) struct Ime {
    output: Option<egui::output::IMEOutput>,
    stage: ImeStage,
    current_text_value: Option<String>,
    current_float_value: Option<f64>,
}

impl Ime {
    pub(crate) fn new() -> Self {
        Self {
            output: None,
            stage: ImeStage::Nothing,
            current_text_value: None,
            current_float_value: None,
        }
    }
    /// Run this before the bottom screen's ctx.run...
    pub(crate) fn handle_input(
        &mut self,
        gfx: &ctru::prelude::Gfx,
        apt: &ctru::prelude::Apt,
        events: &mut Vec<egui::Event>,
    ) {
        if self.output.is_some() && self.stage == ImeStage::Nothing {
            use ctru::applets::swkbd;
            let mut kbd =
                swkbd::SoftwareKeyboard::new(swkbd::Kind::Normal, swkbd::ButtonConfig::LeftRight);
            kbd.set_initial_text(
                self.current_text_value
                    .take()
                    .map(std::borrow::Cow::Owned)
                    .or(self
                        .current_float_value
                        .take()
                        .map(|x| std::borrow::Cow::Owned(x.to_string()))),
            );
            let (text, button) = kbd.launch(apt, gfx).unwrap();
            if button == swkbd::Button::Right {
                self.current_text_value = Some(text);
                self.stage = ImeStage::START;
            } else {
                self.stage = ImeStage::CANCEL;
            }
        }
        if self.stage.add_event(events) {
            events.push(egui::Event::Text(
                self.current_text_value.take().unwrap_or_default(),
            ));
        }
        self.stage.advance();
    }

    /// ...and run this after the bottom screen's ctx.run
    pub(crate) fn handle_output(&mut self, out: &egui::FullOutput) {
        for e in &out.platform_output.events {
            if let egui::output::OutputEvent::Clicked(widget_info) = e
                && self.stage == ImeStage::Nothing
            {
                self.current_text_value = widget_info.current_text_value.clone();
                self.current_float_value = widget_info.value;
            }
        }
        self.output = out.platform_output.ime;
    }
}
