use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );

    println!("Ready to babysit SDDM!");

    loop {
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut proc_iter = sys.processes_by_exact_name("sddm-helper".as_ref());
        let maybe_proc = proc_iter.next();

        if let Some(proc) = maybe_proc {
            let count = proc_iter.count();
            if count > 0 {
                eprintln!("{} more SDDM helper were found! Suspicious…", count)
            }

            println!("Watching helper on pid {}…", proc.pid().as_u32());
            if let Some(ret) = proc.wait() {
                if ret.success() {
                    println!("Helper exited successfuly! SDDM is probably happy!");
                    continue;
                } else {
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
            }
        }
    }
}
