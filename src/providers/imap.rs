use mailparse::{parse_headers, MailHeaderMap};
use thiserror::Error;

use crate::Result;
use crate::types::{MessagePreview};


type ImapSession = imap::Session<native_tls::TlsStream<std::net::TcpStream>>;

#[derive(Error, Debug)]
pub enum HeaderParseError {
    #[error("fetch call didn't include headers")]
    NoHeader,
    #[error("couldn't find `{0}` header")]
    NoHeaderField(&'static str)
}

fn parse_imap_message_preview(message: &imap::types::Fetch) -> Result<MessagePreview> {
    let header_bytes = message.header().ok_or_else(|| HeaderParseError::NoHeader)?;
    let (headers, _) = parse_headers(header_bytes)?;

    let date_header = headers.get_first_value("Date").ok_or_else(|| HeaderParseError::NoHeaderField("Date"))?;
    let timestamp = mailparse::dateparse(&date_header)?;

    let title = headers.get_first_value("Subject").ok_or_else(|| HeaderParseError::NoHeaderField("Subject"))?;
    let author = headers.get_first_value("From").ok_or_else(|| HeaderParseError::NoHeaderField("Date"))?;

    Ok(MessagePreview {title, author, timestamp})
}

pub fn login_imap(config: crate::accounts::ImapAccountConfig) -> Result<ImapSession> {
    let tls = native_tls::TlsConnector::builder().build()?;

    let client = imap::connect((config.domain.as_str(), config.port), &config.domain, &tls)?;

    Ok(client.login(config.email, config.password).map_err(|e| e.0)?)
}

pub fn fetch_range_message_previews(start: usize, stop: usize, imap_session: &mut ImapSession) -> Result<Vec<MessagePreview>> {
    imap_session.select("INBOX")?;

    let mut uids: Vec<_> = imap_session.search("ALL")?.into_iter().collect();
    uids.sort_unstable();

    let most_recent: Vec<_> = uids.iter().rev().skip(start).take(stop).map(|x| x.to_string()).collect();
    let messages = imap_session.fetch(most_recent.join(","), "BODY.PEEK[HEADER]")?;

    messages.iter().rev().map(parse_imap_message_preview).collect()
}