use tui::{
    widgets::{ListState}
};

pub struct FileListing {
    list_state: ListState, 
    file_list: Vec<String>,
    path: String
}

