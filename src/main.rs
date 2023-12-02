use adw::{
    glib,
    gtk::{Align, Box, Orientation},
    prelude::*,
    Application, ApplicationWindow, HeaderBar,
};
use webkit6::{prelude::*, WebView};

const PAGE_URL: &str = "https://thisweek.gnome.org/";
const TITLE: &str = "Matvei 🏳️‍🌈";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.gnome.webkit6-rs.example")
        .build();

    app.connect_activate(|app| {
        set_load_css();

        let webview = web_view();
        let go_back = go_back(&webview);
        let progress = progress_bar(&webview, &go_back);
        let title = window_title(  &webview);
        let header_bar = header_bar(go_back, title);

        let content = Box::new(Orientation::Vertical, 0);

        content.append(&header_bar);
        content.append(&progress);
        content.append(&webview);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Matvei🏳️‍🌈")
            .default_width(800)
            .default_height(800)
            .content(&content)
            .build();

        window.show();
    });
    app.run()
}

fn header_bar(go_back: adw::gtk::Button, title: adw::WindowTitle) -> HeaderBar {
    let header_bar = {
        let header_bar = HeaderBar::new();
        header_bar.pack_start(&go_back);
        header_bar.add_css_class("flat");
        header_bar.set_title_widget(Some(&title));
        header_bar
    };
    header_bar
}

fn set_load_css() {
    if let Some(settings) = adw::gtk::Settings::default() {
        settings.connect_gtk_application_prefer_dark_theme_notify(load_css);
        settings.connect_gtk_theme_name_notify(load_css);
        load_css(&settings);
    }
}

fn window_title(
    webview: &WebView,
) -> adw::WindowTitle {
    let title = adw::WindowTitle::builder()
        .title(TITLE)
        .subtitle(PAGE_URL)
        .build();

    {
        let title = title.clone();

        webview.bind_property("uri", &title, "subtitle").build();
    }
    title
}

fn go_back(webview: &WebView) -> adw::gtk::Button {
    let webview = webview.clone();
    let go_back = adw::gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .sensitive(false)
        .build();

    go_back.connect_clicked(move |_button| {
        webview.go_back();
    });
    go_back
}

fn progress_bar(webview: &WebView, go_back: &adw::gtk::Button) -> adw::gtk::ProgressBar {
    let progress = adw::gtk::ProgressBar::new();
    progress.set_ellipsize(adw::gtk::pango::EllipsizeMode::Start);
    progress.add_css_class("osd");

    {
        let progress = progress.clone();
        webview.connect_estimated_load_progress_notify(move |webview| {
            progress.set_fraction(webview.estimated_load_progress())
        });
    }

    {
        let go_back = go_back.clone();
        let progress = progress.clone();
        webview.connect_load_changed(move |webview, event| match event {
            webkit6::LoadEvent::Started => {}
            webkit6::LoadEvent::Redirected => {}
            webkit6::LoadEvent::Committed => {}
            webkit6::LoadEvent::Finished => {
                let progress = progress.clone();
                glib::MainContext::default().spawn_local(async move {
                    adw::glib::timeout_future(std::time::Duration::from_secs_f32(0.5)).await;
                    progress.set_fraction(0.0);
                });

                if webview.can_go_back() {
                    go_back.set_sensitive(true);
                } else {
                    go_back.set_sensitive(false);
                }
            }
            _ => unreachable!(),
        });
    }

    progress
}

fn web_view() -> WebView {
    let webview = WebView::new();
    webview.load_uri(PAGE_URL);

    webview.set_valign(Align::Fill);
    webview.set_vexpand(true);
    webview.set_halign(Align::Fill);
    webview.set_hexpand(true);

    {
        let settings = webkit6::Settings::new();
        settings.set_enable_developer_extras(true);
        settings.set_enable_write_console_messages_to_stdout(true);

        webview.set_settings(&settings);
    }

    webview
}

fn load_css(settings: &adw::gtk::Settings) {
    let display = adw::gdk::Display::default().expect("Could not get default display.");
    let provider = adw::gtk::CssProvider::new();
    let priority = adw::gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
    let theme_name = settings
        .gtk_theme_name()
        .expect("Could not get theme name.");

    if theme_name.to_lowercase().contains("dark") || settings.is_gtk_application_prefer_dark_theme()
    {
        provider.load_from_data(include_str!("styles/dark.css"));
    } else {
        provider.load_from_data(include_str!("styles/light.css"));
    }

    adw::gtk::style_context_add_provider_for_display(&display, &provider, priority)
}
