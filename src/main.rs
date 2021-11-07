use mailparse::{parse_headers, MailHeaderMap};

mod accounts;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type GenericResult = Result<()>;

#[derive(Debug)]
struct MyError {
    details: String
}

impl MyError {
    fn new(msg: &str) -> MyError {
        MyError{details: msg.to_string()}
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl std::error::Error for MyError {
    fn description(&self) -> &str {
        &self.details
    }
}


struct MessagePreview {
    title: String,
    author: String,
    timestamp: i64
}

impl MessagePreview {
    fn new(title: String, author: String, timestamp: i64) -> MessagePreview {
        MessagePreview {title, author, timestamp}
    }
}

fn parse_message_preview(message: &imap::types::Fetch) -> Result<MessagePreview> {
    let header_bytes = message.header().ok_or_else(|| MyError::new("Didn't fetch a header"))?;
    let (headers, _) = parse_headers(header_bytes)?;

    let date_header = headers.get_first_value("Date").ok_or_else(|| MyError::new("No Date header"))?;
    let timestamp = mailparse::dateparse(&date_header)?;

    let title = headers.get_first_value("Subject").ok_or_else(|| MyError::new("No Subject header"))?;

    let author = headers.get_first_value("From").ok_or_else(|| MyError::new("No From header"))?;

    Ok(MessagePreview::new(title, author, timestamp))
}

type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

fn login_imap(config: accounts::ImapAccountConfig) -> Result<ImapSession> {
    let tls = native_tls::TlsConnector::builder().build()?;

    let client = imap::connect((config.domain.as_str(), config.port), &config.domain, &tls)?;

    Ok(client.login(config.email, config.password).map_err(|e| e.0)?)

}

fn fetch_n_newest_message_previews(n: usize, imap_session: &mut ImapSession) -> Result<Vec<MessagePreview>> {
    imap_session.select("INBOX")?;

    let mut uids: Vec<_> = Vec::from_iter(imap_session.search("ALL")?);
    uids.sort_unstable();

    let most_recent: Vec<_> = uids.iter().rev().take(n).map(|x| x.to_string()).collect();
    let messages = imap_session.fetch(most_recent.join(","), "BODY.PEEK[HEADER]")?;

    messages.iter().rev().map(parse_message_preview).collect()
}

fn main() -> GenericResult {
    let mut imap_session = login_imap(accounts::get_account_config()?)?;

    println!("Logged in!");

    for preview in fetch_n_newest_message_previews(4, &mut imap_session)? {
        println!("Subject: {}\nAuthor: {}\nTimestamp: {}\n", preview.title, preview.author, preview.timestamp);
    }

    imap_session.logout()?;

    Ok(())
}
