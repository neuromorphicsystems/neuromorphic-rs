#[test]
fn convert() {
    let bytes = std::fs::read("tests/test.raw").unwrap();
    let mut events = Vec::new();

    {
        let mut adapter = neuromorphic_drivers::adapters::evt3::Adapter::from_dimensions(1280, 720);
        let start = std::time::Instant::now();
        adapter.convert(&bytes, |event| events.push(event), |_| {});
        println!(
            "convert: {} µs, t = {}",
            start.elapsed().as_micros(),
            adapter.current_t()
        );
    }
    {
        let mut adapter = neuromorphic_drivers::adapters::evt3::Adapter::from_dimensions(1280, 720);
        let start = std::time::Instant::now();
        adapter.consume(&bytes);
        println!(
            "consume: {} µs, t = {}",
            start.elapsed().as_micros(),
            adapter.current_t()
        );
    }

    std::fs::write("tests/test.events", unsafe {
        std::slice::from_raw_parts(
            events.as_ptr() as *const u8,
            events.len() * core::mem::size_of::<neuromorphic_types::DvsEvent<u64, u16, u16>>(),
        )
    })
    .unwrap();
}
