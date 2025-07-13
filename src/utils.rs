use nix::sys::signal::Signal;

pub fn is_signal_deadly(sig: Signal) -> bool {
    match sig {
        Signal::SIGCHLD => false,
        Signal::SIGCONT => false,
        Signal::SIGSTOP => false,
        Signal::SIGTSTP => false,
        Signal::SIGTTIN => false,
        Signal::SIGTTOU => false,
        Signal::SIGURG => false,
        Signal::SIGWINCH => false,
        _ => true,
    }
}
