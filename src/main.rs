extern crate imap;
extern crate native_tls;

use mailparse::{parse_headers, MailHeaderMap};

mod accounts;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type GenericResult = Result<()>;

type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

fn login_imap(config: accounts::ImapAccountConfig) -> Result<ImapSession> {
    let tls = native_tls::TlsConnector::builder().build()?;

    let clone = config.domain.clone();
    let client = imap::connect((config.domain, config.port), clone, &tls)?;

    Ok(client.login(config.email, config.password).map_err(|e| e.0)?)

}

fn fetch_newest_email_subjects(mut imap_session: ImapSession) -> Result<Vec<String>> {
    imap_session.select("INBOX")?;

    let mut uids: Vec<_> = Vec::from_iter(imap_session.search("ALL")?);
    uids.sort();

    let most_recent: Vec<_> = uids.iter().rev().take(4).map(|x| x.to_string()).collect();
    let messages = imap_session.fetch(most_recent.join(","), "BODY.PEEK[HEADER.FIELDS (Subject)]")?;

    let subjects: Vec<_> = messages.iter().rev().map(|message| {
        let header_bytes = message.header().expect("Message did not have a header!");
        let (headers, _) = parse_headers(header_bytes).expect("Could not parse headers");
        headers.get_first_value("Subject").expect("No subject header")
    }).collect();
    
    imap_session.logout()?;

    Ok(subjects)
}

fn main() -> GenericResult {
    let imap_session = login_imap(accounts::get_account_config()?)?;

    for subject in fetch_newest_email_subjects(imap_session)? {
        println!("{}", subject);
    }
    
    Ok(())
}
