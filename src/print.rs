use std::fmt::Display;

use console::{pad_str, style, Alignment, StyledObject};

pub struct Printer {
    pub width: u16,
}

impl Printer {
    pub fn center(&self, message: &str) -> String {
        pad_str(message, self.width.into(), Alignment::Center, None).to_string()
    }

    pub fn print_center<T: Display>(&self, message: T) {
        println!("{}", self.center(&message.to_string()));
    }

    pub fn print_info<T1: Display, T2: Display>(&self, label: T1, info: T2) {
        let left_width = usize::from((self.width / 2) - 1);
        println!("{:>width$}  {}", style(label), info, width = left_width);
    }

    pub fn print_section_heading(&self, heading: &str) {
        self.print_info(style(heading).bold(), "");
    }

    pub fn print_failure(&self, message: &str) {
        println!();
        println!("{}", style(self.center(message)).red());
        println!();
    }

    pub fn print_image(&self, image: &image::DynamicImage, width: u16) -> Result<(), ()> {
        let conf = viuer::Config {
            transparent: true,
            absolute_offset: false,
            x: (self.width - width) / 2,
            y: 0,
            width: Some(width.into()),
            ..Default::default()
        };

        println!();
        match viuer::print(&image, &conf) {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
    }
}

pub fn styled_empty_value() -> StyledObject<String> {
    style(String::from("-")).dim()
}
