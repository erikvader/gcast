use protocol_macros::message_part;

#[message_part]
struct Tree {
    breadcrumbs: Vec<String>,
    contents: Vec<Entry>,
}

#[message_part]
enum Entry {
    File {
        path: String,
        root: usize,
        name: String,
    },
    Dir {
        name: String,
        id: usize,
    },
}
