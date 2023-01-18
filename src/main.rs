use sdomain_test::range_generators::gen_log_range;
use sdomain_test::pdn::PDNModel;
use sdomain_test::passives::capacitor::Capacitor;
use sdomain_test::sdomain::{self, Fs};
use sdomain_test::complex::Complex;



use sdomain_test_plotters::pdn_impedance_plotter::pdn_plotter;
use plotters::{prelude::*, style::full_palette::{PURPLE, GREY}};



fn main() {
    let area_dims = (1600, 600);
    let drawing_area = BitMapBackend::new("images/bodes.png", area_dims)
        .into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();
    let (left, right) = drawing_area.split_horizontally((50).percent_width());

    let zr = sdomain::gen::resistor(100.0);
    let zc = sdomain::gen::capacitor(4.7e-6);
    let lpf = zc.clone() / &(zr + &zc);
    
    let zr_top = sdomain::gen::resistor(300e3);
    let zr_bottom = sdomain::gen::resistor(100e3);
    let zc = sdomain::gen::capacitor(4e-12);
    let hpf = zr_bottom.clone() / &(zr_bottom + &sdomain::parallel(zr_top, zc));

    plot_sdomain(&left, "Low Pass Filter", lpf).unwrap();
    plot_sdomain(&right, "High Pass Filter", hpf).unwrap();

    

    const COLS: u32 = 2;
    const ROWS: u32 = 2;
    let area_dims = (800*COLS, 600*ROWS);
    let drawing_area = BitMapBackend::new("images/component_impedances.png", area_dims)
        .into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();
    let subareas = drawing_area.split_evenly((ROWS as usize, COLS as usize));
    plot_impedance(&subareas[0], "resistor", sdomain::gen::resistor(10.0), None).unwrap();
    plot_impedance(&subareas[1], "capacitor", sdomain::gen::capacitor(22e-6), None).unwrap();
    plot_impedance(&subareas[2], "inductor", sdomain::gen::inductor(1.5e-6), None).unwrap();
    plot_impedance(&subareas[3], "RCL", sdomain::gen::rcl(1e-3, 10e-6, 1.5e-9), None).unwrap();


    let area_dims = (960, 720);
    let drawing_area = BitMapBackend::new("images/pdn_impedance.png", area_dims)
        .into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();

    let mut pdn = PDNModel::from(sdomain::gen::rl(52e-3, 1.5e-6), None);
    pdn.add_capacitor("0603 22uF", Capacitor::from(22e-6, "0603").model(), 1);
    pdn.add_capacitor("0402 10uF", Capacitor::from(10e-6, "0402").model(), 2);
    pdn.add_capacitor("0402 4.7uF", Capacitor::from(4.7e-6, "0402").model(), 2);
    pdn.add_capacitor("0201 2.2nF", Capacitor::from(2.2e-9, "0201").model(), 4);
    pdn.add_capacitor("0201 100nF", Capacitor::from(100e-9, "0201").model(), 3);
    const CENTER: f64 = 25e3;
    const ERR: f64 = 100e3;
    const CENTER_KHZ: f64 = CENTER*1e-3;
    const ERR_KHZ: f64 = ERR*1e-3;
    match Capacitor::from_resonant(CENTER, ERR) {
        Some(c) => {
            let resonant = c.resonant()*1e-3;
            println!("Found cap near {CENTER_KHZ:.0}kHz: {c:}\n  Resonant = {resonant:.0}kHz");
            pdn.add_capacitor(&format!("~{CENTER_KHZ:.0}kHz"), c.model(), 4);
        },
        None => println!("Could not find a cap near {CENTER_KHZ:.0}kHz within {ERR_KHZ:.0}kHz")
    }
    const CENTER2: f64 = 55e6;
    const ERR2: f64 = 5e6;
    const CENTER_MHZ: f64 = CENTER2*1e-6;
    const ERR_MHZ: f64 = ERR2*1e-6;
    match Capacitor::from_resonant(CENTER2, ERR2) {
        Some(c) => {
            let resonant = c.resonant()*1e-6;
            println!("Found cap near {CENTER_MHZ:.0}MHz: {c:}\n  Resonant = {resonant:.0}MHz");
            pdn.add_capacitor(&format!("~{CENTER_MHZ:.0}MHz"), c.model(), 1);
        },
        None => println!("Could not find a cap near {CENTER_MHZ:.0}MHz within {ERR_MHZ:.0}MHz")
    }
    pdn_plotter::plot(&pdn, &drawing_area, Some(0.1)).unwrap();
    println!("Miscellaenous done!");
}



type DrawAreaType<'a> = DrawingArea <BitMapBackend<'a>, plotters::coord::Shift>;
    
fn plot_sdomain(drawing_area: &DrawAreaType, name: &str, fs: Fs) -> Result<(), Box <dyn std::error::Error>> {
    let freq_data = gen_log_range(1.0, 10.0e6, 10.0, 100);
    let complex_data = freq_data.iter().map(|freq| fs.calculate_freq(*freq)).collect::<Vec<Complex>>();
    let mag_data = complex_data.iter().map(|c| c.mag_20log()).collect::<Vec<f64>>();
    let phase_data = complex_data.iter().map(|c| c.phase_deg()).collect::<Vec<f64>>();

    let mut max_mag = 0.0;
    for mag in mag_data.iter() {if max_mag < *mag {max_mag = *mag;}}
    max_mag += 1.0;
    let mut min_mag = 1e12;
    for mag in mag_data.iter() {if min_mag > *mag {min_mag = *mag;}}
    min_mag -= 1.0;

    let mut chart = ChartBuilder::on(&drawing_area)
    .caption(format!("Bode Plot for {name}"), ("Arial", 30))
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .margin(10)
        .build_cartesian_2d((1.0f64..10_000_000f64).log_scale(), min_mag..max_mag)
        .unwrap()
        .set_secondary_coord((1.0f64..10_000_000f64).log_scale(), -180.0..180.0);

    chart.configure_mesh().x_desc("Frequency [Hz]").y_desc("Magnitude [dB]").draw().unwrap();
    chart.configure_secondary_axes().x_desc("Frequency [Hz]").y_desc("Phase [°]").draw().unwrap();

    let freq_mag_iter = freq_data.clone().into_iter().zip(mag_data);
    let freq_phase_iter = freq_data.into_iter().zip(phase_data);

    chart.draw_series(LineSeries::new(
            freq_mag_iter,
            &GREEN
        ))
        .unwrap()
        .label("Magnitude")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &GREEN));

    chart.draw_secondary_series(LineSeries::new(
            freq_phase_iter,
            &RED.mix(0.4)
        ))
        .unwrap()
        .label("Phase")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));


    chart.configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(&GREY.mix(0.3))
        .draw()
        .unwrap();

    Ok(())
}

fn plot_impedance(drawing_area: &DrawAreaType, name: &str, fs: Fs, impedance_target: Option<f64>) -> Result<(), Box <dyn std::error::Error>> {
    let freq_data = gen_log_range(1.0, 10.0e6, 10.0, 100);
    let complex_data = freq_data.iter().map(|freq| fs.calculate_freq(*freq)).collect::<Vec<Complex>>();
    let mag_data = complex_data.iter().map(|c| c.mag()).collect::<Vec<f64>>();
    let phase_data = complex_data.iter().map(|c| c.phase_deg()).collect::<Vec<f64>>();

    let mut min_mag = 1e12;
    for mag in mag_data.iter() {if min_mag > *mag {min_mag = *mag;}}
    min_mag *= 1e4;

    let mut chart = ChartBuilder::on(&drawing_area)
    .caption(format!("Impedance of {name}"), ("Arial", 30))
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .margin(10)
        .build_cartesian_2d((1.0f64..10_000_000f64).log_scale(), (0.0..min_mag).log_scale())
        .unwrap()
        .set_secondary_coord((1.0f64..10_000_000f64).log_scale(), -180.0..180.0);

    chart.configure_mesh().x_desc("Frequency [Hz]").y_desc("Impedance [Ω]").draw().unwrap();
    chart.configure_secondary_axes().x_desc("Frequency [Hz]").y_desc("Phase [°]").draw().unwrap();

    let freq_mag_iter = freq_data.clone().into_iter().zip(mag_data);
    let freq_phase_iter = freq_data.into_iter().zip(phase_data);

    match impedance_target {
        Some(res) => {
            chart.draw_series(AreaSeries::new(
                    freq_mag_iter,
                    res,
                    &YELLOW.mix(0.3)
                )
                .border_style(&PURPLE))
                .unwrap();
        },
        None => {
            chart.draw_series(LineSeries::new(
                    freq_mag_iter,
                    &GREEN
                ))
                .unwrap()
                .label("Impedance")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &GREEN));
        },
    }
    chart.draw_secondary_series(LineSeries::new(
            freq_phase_iter,
            &RED.mix(0.4)
        ))
        .unwrap()
        .label("Phase")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));



    chart.configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(&GREY.mix(0.3))
        .draw()
        .unwrap();

    Ok(())
}