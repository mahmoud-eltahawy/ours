use super::nav::UnitKind;
use common::assets::IconName;

use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub enum SelectedState {
    Copy,
    Cut,
    #[default]
    None,
}

#[derive(Default, Clone, Debug)]
pub struct Selected {
    pub on: bool,
    pub units: Vec<Unit>,
    pub state: SelectedState,
}

impl Selected {
    pub fn clear(&mut self) {
        self.units.clear();
        self.none();
        self.on = false;
    }

    pub fn as_paths(&self) -> Vec<PathBuf> {
        self.units.iter().map(|x| x.path.clone()).collect()
    }

    pub fn has_dirs(&self) -> bool {
        self.units
            .iter()
            .any(|x| matches!(x.kind, UnitKind::Folder))
    }

    pub fn is_clear(&self) -> bool {
        self.units.is_empty()
    }

    pub fn copy(&mut self) {
        self.state = SelectedState::Copy;
    }

    pub fn cut(&mut self) {
        self.state = SelectedState::Cut;
    }

    pub fn none(&mut self) {
        self.state = SelectedState::None;
    }

    pub fn remove_unit(&mut self, unit: &Unit) {
        self.units.retain(|x| x != unit);
        if self.units.is_empty() {
            self.none();
        }
    }

    pub fn toggle_unit_selection(&mut self, unit: &Unit) {
        if self.units.contains(unit) {
            self.remove_unit(unit);
        } else {
            self.units.push(unit.clone());
        }
    }

    pub fn toggle_unit_alone_selection(&mut self, unit: &Unit) {
        if self.units.contains(unit) {
            self.units.clear();
        } else {
            self.units = vec![unit.clone()];
        }
    }

    pub fn is_selected(&self, unit: &Unit) -> bool {
        self.units.contains(unit)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Unit {
    pub path: PathBuf,
    pub kind: UnitKind,
}

impl From<UnitKind> for IconName {
    fn from(value: UnitKind) -> Self {
        match value {
            UnitKind::Folder => IconName::Folder,
            UnitKind::Video => IconName::Video,
            UnitKind::Audio => IconName::Audio,
            UnitKind::File => IconName::File,
        }
    }
}

impl Unit {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn icon(&self) -> &'static [u8] {
        IconName::from(self.kind).get()
    }
}
