use std::path::PathBuf;

pub struct DialogApi;

impl DialogApi {
    pub fn open_file(title: &str, filters: &[(&str, &[&str])]) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new().set_title(title);
        for &(name, extensions) in filters {
            dialog = dialog.add_filter(name, extensions);
        }
        dialog.pick_file()
    }

    pub fn save_file(title: &str, default_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(title)
            .set_file_name(default_name)
            .save_file()
    }

    pub fn message(title: &str, description: &str) {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(description)
            .show();
    }

    pub fn confirm(title: &str, description: &str) -> bool {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(description)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
    }
}
