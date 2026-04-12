use anyhow::{Context, Result};

mod build;
mod defaults;
mod fetch;
mod map;
mod settings;
mod types;

use crate::settings::get_settings;

fn main() -> Result<()> {
    print!(
        r#"
       .~@`,
      (__,  \
          \' \
           \  \
            \  \
             \  `._            __.__
              \    ~-._  _.==~~     ~~--.._
               \        '                  ~-.
                \      _-   -_                `.
                 \    /       )        .-    .  \
                  `. |      /  )      (       ;  \
                    `|     /  /       (       :   '\
                     \    |  /        |      /       \
                      |     /`-.______\.     |~-.      \
                      |   |/           (     |   `.      \_
                      |   ||            ~\   \      '._    `-.._____..----..___
                      |   |/             _\   \         ~-.__________.-~~~~~~~~~'''
      post_notes    .o'___/            .o______)


        "#
    );

    colog::init();

    log::info!("=== Loading Settings ===");
    let settings = get_settings();

    println!();

    log::info!(
        "=== Starting to load content from {}. ===",
        &settings.path.input.display()
    );
    let raw = fetch::notes(&settings.path.input).context("Failed to read content")?;
    let notes = map::notes(raw);

    println!();

    log::info!(
        "=== Starting to generate content map with {} entrie(s). ===",
        notes.len()
    );
    let content = map::content(&notes);

    println!();

    log::info!("=== Starting to generate navigation. ===");
    let nav = map::navigation(&notes);

    println!();

    log::info!("=== Starting to build website. ===");
    build::website(&notes, content, nav, &settings).context("Failed to build website")?;

    Ok(())
}
