pub fn get_os_timer_freq() -> u64 {
    unsafe {
        let mut freq = std::mem::zeroed();
        winapi::um::profileapi::QueryPerformanceFrequency(&mut freq);
        *freq.QuadPart() as u64
    }
}

pub fn read_os_timer() -> u64 {
    unsafe {
        let mut value = std::mem::zeroed();
        winapi::um::profileapi::QueryPerformanceCounter(&mut value);
        *value.QuadPart() as u64
    }
}

pub fn read_cpu_timer() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn estimate_cpu_timer_freq(milliseconds_to_wait: Option<u64>) -> u64 {
    let os_freq = get_os_timer_freq();

    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_elapsed: u64 = 0;
    let os_wait_time = os_freq * milliseconds_to_wait.unwrap_or(1000) / 1000;
    while os_elapsed < os_wait_time {
        os_elapsed = read_os_timer() - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let cpu_freq: u64 = if os_elapsed > 0 {
        os_freq * cpu_elapsed / os_elapsed
    } else {
        0
    };

    cpu_freq
}
