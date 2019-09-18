use azul::prelude::*;
use azul::widgets::text_input::*;

use crate::errors::AppError;



struct AppDataModel {
    query: TextInputState
}


impl Layout for AppDataModel {
    fn layout(&self, info: LayoutInfo<Self>) -> Dom<Self> {
        TextInput::new()
        .bind(info.window, &self.query, &self)
        .dom(&self.query)
        .with_id("query")
    }
}

impl Default for AppDataModel {
    fn default() -> Self {
        Self {
            query: TextInputState::new("foo")
        }
    }
}

pub fn main() -> Result<(), AppError> {
    let mut app = App::new(AppDataModel::default(), AppConfig::default()).unwrap();
    let window = app.create_window(WindowCreateOptions::default(), css::native()).unwrap();
    app.run(window).unwrap();

    Ok(())
}
