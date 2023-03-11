use protocol::{
    to_client::front::errormsg,
    to_server::{errormsgctrl::ErrorMsgCtrl, ToServer},
};

use super::{Control, MachineResult, StateLogger};

pub(super) async fn error_msg_state(
    ctrl: &mut Control,
    header: String,
    body: String,
) -> MachineResult<()> {
    let logger = StateLogger::new("UserError");
    let state = errormsg::ErrorMsg { header, body };

    while let Some(msg) = ctrl.send_recv(state.clone()).await {
        match msg {
            ToServer::ErrorMsgCtrl(ErrorMsgCtrl::Close) => break,
            _ => logger.invalid_message(&msg),
        }
    }

    Ok(())
}
