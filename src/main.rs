// Copyright (c) 2025 tsrk. <tsrk@tsrk.me>
// This file is licensed under the MIT license
// See the LICENSE file in the repository root for more info.

// SPDX-License-Identifier: MIT

use nix::{
    sys::{
        ptrace::{self, Options},
        signal::{self, Signal},
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{mem, thread, time::Duration};
use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

const MISSING_THRES: u8 = 10;

fn kill_elon<'a>(sys: &'a System) {
    sys.processes_by_exact_name("X".as_ref())
        .filter(|x| {
            if let Some(parent_pid) = x.parent() {
                if let Some(parent) = sys.process(parent_pid) {
                    return parent.name() == "sddm";
                }
            }
            false
        })
        .for_each(|x| {
            let xpid = x.pid().as_u32();
            if x.kill() {
                println!("Killing process {}", xpid);
            } else {
                eprintln!("Failed to kill process {}", xpid);
            }
        });
}

fn babysit<'a>(ret: i32, sys: &'a System) {
    if ret == 0 {
        println!("Helper exited successfuly! SDDM is probably happy!");
        return;
    }
    println!("Oh no! Helper died tragically, SDDM will cry!");
    println!("Killing X server to make it happy again…");
    kill_elon(sys);
}

fn dispatch<'a>(status: WaitStatus, sys: &'a System) -> bool {
    match status {
        WaitStatus::Exited(_pid, ret) => {
            babysit(ret, &sys);
            true
        }
        WaitStatus::Stopped(r_pid, sig) => {
            println!("Process {r_pid:?} got stopped by {sig:?}");
            if let Err(e) = signal::kill(r_pid, Signal::SIGCONT) {
                eprintln!("Failed to send SIGCONT to {r_pid:?}: {}", e.desc());
            }
            false
        }
        WaitStatus::PtraceEvent(r_pid, sig, ev) => {
            let event: ptrace::Event = unsafe { mem::transmute(ev as i32) };
            println!("Received event {event:?}, with signal {sig:?} attempting to reinject signal");
            if let Err(e) = ptrace::cont(r_pid, sig) {
                let i32_pid = r_pid.as_raw();
                eprintln!("Failed to continue {i32_pid}: {}", e.desc());
            }
            false
        }
        WaitStatus::Signaled(_pid, sig, _cd) => {
            println!("Helper received signal {sig:?}");
            false
        }
        x => {
            eprintln!("{:?}", x);
            false
        }
    }
}

fn watch_helper<'a>(proc: &Process, sys: &'a System) {
    if let Ok(i32_pid) = i32::try_from(proc.pid().as_u32()) {
        let n_pid = Pid::from_raw(i32_pid);

        if let Err(e) = ptrace::seize(n_pid, Options::PTRACE_O_TRACESYSGOOD) {
            eprintln!("Failed to trace process {i32_pid}: {}", e.desc());
            return;
        }
        println!("Watching helper on pid {}…", i32_pid);

        loop {
            match waitpid(n_pid, None) {
                Ok(status) => {
                    if dispatch(status, sys) {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to wait for process {i32_pid}: {}", e.desc());
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    let mut missing_helper_count: u8 = 0;

    println!("Ready to babysit SDDM!");

    let mut last_pid: Option<u32> = None;

    loop {
        thread::sleep(Duration::from_millis(1500));
        sys.refresh_processes(ProcessesToUpdate::All, true);

        println!("Searching for helper process…");

        let mut proc_iter = sys.processes_by_exact_name("sddm-helper".as_ref());
        let mut helper_found = false;

        while let Some(proc) = proc_iter.next() {
            if Some(proc.pid().as_u32()) == last_pid {
                eprintln!("Skipping old helper!");
                continue;
            }
            helper_found = true;
            missing_helper_count = 0;
            last_pid = Some(proc.pid().as_u32());
            println!("Found helper at pid {}", proc.pid().as_u32());
            watch_helper(&proc, &sys);
            break;
        }

        if helper_found {
            continue;
        }

        missing_helper_count += 1;
        if missing_helper_count > MISSING_THRES {
            println!("No SDDM helper?! Maybe it already died!");
            kill_elon(&sys);
            missing_helper_count = 0;
        }
    }
}
