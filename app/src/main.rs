pub mod user_interface;
use user_interface::MyState;
use user_interface::MyUIDelegate;

use ui::application::Application;
use ui::ui_application_delegate::UIApplicationDelegate;

fn main() {
    let app: Application<MyState> = Application::new("My Application");
    let app_delegate =
        UIApplicationDelegate::new().with_window("My Window", 1200, 800, Box::new(MyUIDelegate {}));

    app.run(Box::new(app_delegate), MyState { count: 0 });
}
