use protocol::{
    to_client::front::errormsg,
    to_server::{errormsgctrl::ErrorMsgCtrl, ToServer},
};

use super::{
    log_invalid_msg, log_state_entered, log_state_exited, Control, MachineResult,
};

pub(super) async fn error_msg_state(
    ctrl: &mut Control,
    header: String,
    body: String,
) -> MachineResult<()> {
    const NAME: &str = "UserError";
    log_state_entered(NAME);

    let state = errormsg::ErrorMsg { header, body };

    while let Some(msg) = ctrl.send_recv(state.clone()).await {
        match msg {
            ToServer::ErrorMsgCtrl(ErrorMsgCtrl::Close) => break,
            _ => log_invalid_msg(NAME, &msg),
        }
    }

    log_state_exited(NAME);
    Ok(())
}
