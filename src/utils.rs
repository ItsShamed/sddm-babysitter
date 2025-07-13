use nix::sys::signal::Signal;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

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

pub fn create_proc_sys() -> System {
    System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()))
}
