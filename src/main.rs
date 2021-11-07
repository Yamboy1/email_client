use std::sync::{Mutex, Arc};
use warp::Filter;
use tera::{Tera, Context};

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

    let mutex = Arc::new(Mutex::new(providers::imap::login_imap(accounts::get_account_config()?)?));

    println!("Logged in");

    let root = warp::path!("/")
        .map(move || {
            let mut imap_session = mutex.lock().unwrap();
            let message_previews = providers::imap::fetch_range_message_previews(0,4, &mut imap_session).unwrap();

            let mut context = Context::new();
            context.insert("previews", &message_previews);

            let content = tera.render("message_preview.html", &context).unwrap();
            warp::reply::html(content)
        });

    warp::serve(root)
        .run(([127,0,0,1], 3030))
        .await;

    Ok(())
}
