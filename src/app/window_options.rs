use display_info::DisplayInfo;

#[derive(Debug)]
pub struct WindowAttributes {
    pub width: u32,
    pub height: u32,
    pub pos_x: i32,
    pub pos_y: i32,
}

impl WindowAttributes {
    pub fn new() -> Self {
        let display_infos = match DisplayInfo::all() {
            Ok(display_infos) => display_infos,
            Err(err) => {
                println!("Failed to get display informations");
                println!("{:?}", err);
                std::process::exit(1);
            }
        };
        let primary_displays = display_infos
            .iter()
            .filter(|display_info| display_info.is_primary)
            .collect::<Vec<&DisplayInfo>>();
        let primary_display_info = match primary_displays.first() {
            Some(display_info) => display_info,
            None => {
                println!("Failed to locate primary display. {:?}", display_infos);
                std::process::exit(1);
            }
        };

        let display_width = primary_display_info.width;
        let display_height = primary_display_info.height;

        let width = (display_width as f32 * 0.75) as u32;
        let height = (display_height as f32 * 0.75) as u32;
        let pos_x = ((display_width - width) / 2) as i32;
        let pos_y = ((display_height - height) / 2) as i32;

        Self {
            width,
            height,
            pos_x,
            pos_y,
        }
    }
}
