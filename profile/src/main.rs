fn main() {
    let mut milliseconds_to_wait: u64 = 1000;

    if std::env::args().count() == 2 {
        milliseconds_to_wait = std::env::args()
            .nth(1)
            .unwrap()
            .parse()
            .expect("Failed to parse milliseconds to wait");
    }

    let os_freq = profile::get_os_timer_freq();
    println!("    OS Freq: {} (reported)", os_freq);

    let cpu_start = profile::read_cpu_timer();
    let os_start = profile::read_os_timer();
    let mut os_end: u64 = 0;
    let mut os_elapsed: u64 = 0;
    let os_wait_time = milliseconds_to_wait * (os_freq / 1000);
    while os_elapsed < os_wait_time {
        os_end = profile::read_os_timer();
        os_elapsed = os_end - os_start;
    }

    let cpu_end = profile::read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let cpu_freq: u64 = if os_elapsed > 0 {
        cpu_elapsed * (os_freq / os_elapsed)
    } else {
        0
    };

    println!(
        "   OS Timer: {} -> {} = {} elapsed",
        os_start, os_end, os_elapsed
    );
    println!(" OS Seconds: {}", os_elapsed as f64 / os_freq as f64);

    println!(
        "  CPU Timer: {} -> {} = {} elapsed",
        cpu_start, cpu_end, cpu_elapsed
    );
    println!(
        "   CPU Freq: {}, {:.2}GHz (guessed)",
        cpu_freq,
        cpu_freq as f64 / 1000000000_f64
    );
}
