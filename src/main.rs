use std::sync::{Mutex, Arc};
use warp::Filter;
use tera::{Tera, Context, Function, Value};


mod accounts;
mod providers;
mod types;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type GenericResult = Result<()>;

#[tokio::main]
async fn main() -> GenericResult {
    let tera = match Tera::new("src/templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    println!("Logging in...");

    let mut a = providers::imap::login_imap(accounts::get_account_config()?)?;

    println!("Logged in!");

    providers::imap::fetch_range_message_previews(0, 4, &mut a).unwrap();

    let mutex = Arc::new(Mutex::new(a));

    let app = warp::path!("app")
        .map(move || {
            let mut imap_session = mutex.lock().unwrap();
            let message_previews = providers::imap::fetch_range_message_previews(0, 20, &mut imap_session).unwrap();

            let mut context = Context::new();
            context.insert("previews", &message_previews);

            let content = tera.render("homepage.html", &context).unwrap();
            warp::reply::html(content)
        });

    let public = warp::path("public")
        .and(warp::fs::dir("src/public"));

    let routes = warp::get().and(
        app.or(public)
    );

    warp::serve(routes)
        .run(([127,0,0,1], 3030))
        .await;

    Ok(())
}
