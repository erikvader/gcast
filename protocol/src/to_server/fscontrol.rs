use crate::ToServer;

// TODO: move to separate files? Like Mpv
pub mod search_ctrl {
    #[protocol_macros::message_aggregator]
    enum SearchCtrl {
        Search(String),
    }
}

pub mod tree_ctrl {
    #[protocol_macros::message_aggregator]
    enum TreeCtrl {
        Cd(usize),
        CdDotDot,
    }
}

#[protocol_macros::message_aggregator(ToServer)]
enum FsControl {
    SearchCtrl(search_ctrl::SearchCtrl),
    TreeCtrl(tree_ctrl::TreeCtrl),
}
