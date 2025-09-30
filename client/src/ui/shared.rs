use iced::{Element, Length, Size, Color};
use iced_widget::{Image, image::Handle, container, text, row, Space};
use crate::messages::Msg;
use crate::App;

pub fn brand_logo() -> Element<'static, Msg> {
    // Embed the PNG at compile time to avoid path issues.
    // Adjust the path to where the file actually lives in your repo.
    const BYTES: &[u8] = include_bytes!("../../../client/assets/cctmog_logo.png");

    let logo: Element<_> = Image::new(Handle::from_path("client/assets/cctmog_logo.png"))
        .width(Length::Fixed(220.0))
        .into();
    logo
}

pub fn footer(app: &App, window_size: Option<Size>) -> Element<'_, Msg> {
    let theme_name = "Dark"; // Since we're using Theme::Dark in main.rs

    let websocket_status = if app.connected {
        format!("🟢 Connected to {}", app.url)
    } else if app.connecting {
        format!("🟡 Connecting to {}", app.url)
    } else {
        "🔴 Disconnected".to_string()
    };

    let screen_info = if let Some(size) = window_size {
        format!("📱 {}×{}", size.width as u32, size.height as u32)
    } else {
        "📱 Size unknown".to_string()
    };

    let user_info = if !app.name.is_empty() {
        format!("👤 {}", app.name)
    } else {
        "👤 No name set".to_string()
    };

    let host_info = if let Some(host_name) = &app.host_name {
        if let Some(port) = app.host_server_port {
            format!("🏠 Host: {} (:{}) {}", host_name, port, if app.is_hosting { "👑" } else { "" })
        } else {
            format!("🏠 Host: {}", host_name)
        }
    } else if app.is_hosting {
        "🏠 You are hosting 👑".to_string()
    } else {
        "🏠 No host".to_string()
    };

    let footer_content = row![
        text(user_info)
            .size(12)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                ..Default::default()
            }),
        Space::with_width(Length::Fixed(20.0)),
        text(host_info)
            .size(12)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                ..Default::default()
            }),
        Space::with_width(Length::Fixed(20.0)),
        text(screen_info)
            .size(12)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                ..Default::default()
            }),
        Space::with_width(Length::Fixed(20.0)),
        text(websocket_status)
            .size(12)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                ..Default::default()
            }),
        Space::with_width(Length::Fixed(20.0)),
        text(format!("🎨 Theme: {}", theme_name))
            .size(12)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                ..Default::default()
            }),
    ]
    .spacing(10);

    container(footer_content)
        .padding([8, 16])
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.09))),
            border: iced::Border {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                width: 1.0,
                radius: iced::border::Radius::from(0.0),
            },
            ..Default::default()
        })
        .into()
}