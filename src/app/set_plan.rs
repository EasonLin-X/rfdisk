use crate::{
    app::state::{App, AppLayer, Focus, InputMode},
    util::i18n::{tr, Lang, Msg},
};

impl App {
    pub(crate) fn enter_set_plan(&mut self) {
        if self.selected_disk().is_none() {
            self.status = tr(self.lang, Msg::NoSelectedDisk).to_string();
            return;
        };

        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 2;
        self.input_mode = InputMode::SetPlan;
        self.status = match self.lang {
            Lang::En => "Set(plan) is still under development.".to_string(),
            Lang::ZhCn => "设置计划还在研发中。".to_string(),
        };
    }

    pub(crate) fn close_set_plan(&mut self) {
        self.input_mode = InputMode::Normal;
        self.layer = AppLayer::Main;
        self.focus = Focus::Disks;
        self.selected_menu = 2;
        self.status = match self.lang {
            Lang::En => "Set(plan) closed. It is still under development.".to_string(),
            Lang::ZhCn => "设置计划已关闭。该功能还在研发中。".to_string(),
        };
    }
}
