pub mod pdn_plotter {
    use plotters::prelude::*;
    use plotters::style::full_palette::{PURPLE, GREY};
    
    use sdomain_test::pdn::PDNModel;
    use sdomain_test::sdomain::Fs;
    use sdomain_test::complex::Complex;
    use sdomain_test::range_generators::gen_log_range;
    type DrawAreaType<'a> = DrawingArea <BitMapBackend<'a>, plotters::coord::Shift>;

    /// Plot a PDN's impedance model over frequency. Optionally, an impedance
    /// target can be specified to highlight at what frequencies the PDN
    /// model exceeds the target.
    /// Returns a result to indicate if the function executed without error.
    /// 
    /// # Arguments
    /// * `model` - A power distribution network model.
    /// * `canvas` - A Plotter's DrawingArea on which to draw the impedance plot.
    /// * `impedance_target` - (Optional) If specified, the data is plotted
    ///                                   as an area curve to show at which
    ///                                   frequencies the impedance exceeds
    ///                                   the target.
    /// 
    /// # Examples
    /// ```
    /// use sdomain_test_plotters::pdn_impedance_plotter::pdn_plotter;
    /// 
    /// use plotters::prelude::*;
    /// use sdomain_test::passives::capacitor::Capacitor;
    /// use sdomain_test::pdn::PDNModel;
    /// use sdomain_test::sdomain;
    ///
    /// // Prepare the drawing area
    /// let area_dims = (960, 720);
    /// let drawing_area = BitMapBackend::new("images/pdn_impedance.png", area_dims)
    ///     .into_drawing_area();
    ///
    /// drawing_area.fill(&WHITE).unwrap();
    ///
    /// // Prepare the PDN network to plot
    /// let mut pdn = PDNModel::from(sdomain::gen::rl(52e-3, 1.5e-6), None);
    /// pdn.add_capacitor("0603 22uF", Capacitor::from(22e-6, "0603").model(), 1);
    /// pdn.add_capacitor("0201 2.2nF", Capacitor::from(2.2e-9, "0201").model(), 4);
    /// pdn.add_capacitor("0201 100nF", Capacitor::from(100e-9, "0201").model(), 3);
    /// pdn.add_capacitor("~25kHz", Capacitor::from_resonant(25e3, 100e3).unwrap().model(), 1);
    /// 
    /// // Plot the PDN network and set 100mΩ as the target impedance.
    /// pdn_plotter::plot(&pdn, &drawing_area, Some(0.1/*Ω*/)).unwrap();
    /// ```
    pub fn plot(model: &PDNModel, canvas: &DrawAreaType, impedance_target: Option<f64>) -> Result<(), Box <dyn std::error::Error>> {
        draw(canvas, "PDN", model.model(), impedance_target)
    }

    /// Plot an s-domain model as impedance over frequency. Optionally, an impedance
    /// target can be specified to highlight at what frequencies the model exceeds
    /// the target.
    /// Returns a result to indicate if the function executed without error.
    /// 
    /// # Arguments
    /// * `canvas` - A Plotter's DrawingArea on which to draw the impedance plot.
    /// * `name` - Model name to print in the plot title.
    /// * `model` - An impedance model in the s-domain.
    /// * `impedance_target` - (Optional) If specified, the data is plotted
    ///                                   as an area curve to show at which
    ///                                   frequencies the impedance exceeds
    ///                                   the target.
    fn draw(canvas: &DrawAreaType, name: &str, model: Fs, impedance_target: Option<f64>) -> Result<(), Box <dyn std::error::Error>> {
        const MAX_FREQ: f64 = 100e6;
        let freq_data = gen_log_range(1.0, MAX_FREQ, 10.0, 100);
        let complex_data = freq_data.iter().map(|freq| model.calculate_freq(*freq)).collect::<Vec<Complex>>();
        let mag_data = complex_data.iter().map(|c| c.mag()).collect::<Vec<f64>>();
        let phase_data = complex_data.iter().map(|c| c.phase_deg()).collect::<Vec<f64>>();

        let mut min_mag = 1e12;
        for mag in mag_data.iter() {if min_mag > *mag {min_mag = *mag;}}
        min_mag *= 1e4;

        let mut chart = ChartBuilder::on(&canvas)
        .caption(format!("Impedance of {name}"), ("Arial", 30))
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Right, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d((1.0f64..MAX_FREQ).log_scale(), (0.0..min_mag).log_scale())
            .unwrap()
            .set_secondary_coord((1.0f64..MAX_FREQ).log_scale(), -180.0..180.0);

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
                    .unwrap()
                    .label("Impedance")
                    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &PURPLE));
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
}