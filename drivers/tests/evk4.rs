use neuromorphic_drivers::device::Usb;

#[test]
fn read() -> Result<(), neuromorphic_drivers::devices::prophesee_evk4::Error> {
    let error_flag: neuromorphic_drivers::error::Flag<
        neuromorphic_drivers::devices::prophesee_evk4::Error,
    > = neuromorphic_drivers::error::Flag::new();
    let event_loop = std::sync::Arc::new(neuromorphic_drivers::usb::EventLoop::new(
        std::time::Duration::from_millis(100),
        error_flag.clone(),
    )?);
    let mut device = neuromorphic_drivers::devices::prophesee_evk4::Device::open(
        &None,
        neuromorphic_drivers::devices::prophesee_evk4::Device::PROPERTIES.default_configuration,
        &neuromorphic_drivers::devices::prophesee_evk4::Device::DEFAULT_USB_CONFIGURATION,
        event_loop,
        error_flag.clone(),
    )?;
    let start = std::time::Instant::now();
    let mut previous = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(10) {
        let buffer_view = device.next_with_timeout(&std::time::Duration::from_millis(100));
        if let Some(buffer_view) = buffer_view {
            let now = std::time::Instant::now();
            println!(
                "{} B (backlog: {} packets, delay: {} µs, data rate: {:.3} MB/s)",
                buffer_view.slice.len(),
                buffer_view.backlog(),
                buffer_view.delay().as_micros(),
                (buffer_view.slice.len() as f64 / 1e6) / now.duration_since(previous).as_secs_f64()
            );
            previous = now;
        }
    }
    Ok(())
}
