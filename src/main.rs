// Copyright (c) 2025 tsrk. <tsrk@tsrk.me>
// This file is licensed under the MIT license
// See the LICENSE file in the repository root for more info.

// SPDX-License-Identifier: MIT

use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{thread, time::Duration};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

const MISSING_THRES: u8 = 10;

fn babysit<'a>(ret: i32, sys: &'a System) {
    if ret == 0 {
        println!("Helper exited successfuly! SDDM is probably happy!");
        return;
    }
    println!("Oh no! Helper died tragically, SDDM will cry!");
    println!("Killing X server to make it happy again…");

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

fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    let mut missing_helper_count: u8 = 0;

    println!("Ready to babysit SDDM!");

    loop {
        thread::sleep(Duration::from_millis(1500));
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut proc_iter = sys.processes_by_exact_name("sddm-helper".as_ref());
        let maybe_proc = proc_iter.next();

        if let Some(proc) = maybe_proc {
            missing_helper_count = 0;
            let count = proc_iter.count();
            if count > 0 {
                eprintln!("{} more SDDM helper were found! Suspicious…", count)
            }

            if let Ok(i32_pid) = i32::try_from(proc.pid().as_u32()) {
                let n_pid = Pid::from_raw(i32_pid);

                if let Err(e) = ptrace::attach(n_pid) {
                    eprintln!("Failed to trace process {i32_pid}: {}", e.desc());
                    continue;
                }
                println!("Watching helper on pid {}…", i32_pid);

                loop {
                    match waitpid(n_pid, None) {
                        Ok(status) => match status {
                            WaitStatus::Exited(_pid, ret) => {
                                babysit(ret, &sys);
                                break;
                            }
                            WaitStatus::Stopped(_pid, sig) => match sig {
                                Signal::SIGSTOP => println!("Ptrace SIGSTOP ok"),
                                _ => {
                                    eprintln!("Process got stopped by {sig:?}");
                                }
                            },
                            x => {
                                eprintln!("{:?}", x);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to wait for process {}: {}", i32_pid, e.desc());
                            break;
                        }
                    }
                }
            }
        } else {
            missing_helper_count += 1;
            if missing_helper_count > MISSING_THRES {
                println!("No SDDM helper?! Mayber it already died!");
                babysit(-1, &sys);
                missing_helper_count = 0;
            }
        }
    }
}
