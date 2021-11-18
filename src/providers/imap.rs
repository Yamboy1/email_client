use mailparse::{addrparse, parse_headers, MailAddr, MailHeaderMap};
use thiserror::Error;
use chrono::{Local, TimeZone};
use scraper::{Html, Selector};

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
    let parsed = mailparse::parse_mail(message.body().unwrap()).unwrap();

    let title = parsed.headers.get_first_value("Subject").ok_or_else(|| HeaderParseError::NoHeaderField("Subject"))?;

    let date_header = parsed.headers.get_first_value("Date").ok_or_else(|| HeaderParseError::NoHeaderField("Date"))?;
    let timestamp = mailparse::dateparse(&date_header)?;
    let date_time = Local.timestamp(timestamp, 0);
    let time = date_time.to_rfc3339();

    let from_header = parsed.headers.get_first_value("From").ok_or_else(|| HeaderParseError::NoHeaderField("Date"))?;
    let parsed_header = &addrparse(&from_header).unwrap()[0];
    let author = match parsed_header {
        MailAddr::Single(info) => {
            info.display_name.as_ref().unwrap_or(&info.addr)
        }
        _ => panic!()
    };
    let author = String::from(author);
    let mut text = parsed.get_body().unwrap();
    if parsed.ctype.mimetype == "multipart/alternative" {
        println!("body = {}", parsed.get_body().unwrap());
        for part in parsed.subparts {
            if part.ctype.mimetype == "text/plain" {
                println!("sub mime = {}", part.ctype.mimetype);
                println!("sub body = {}", part.get_body().unwrap());
                text = part.get_body().unwrap();
            }
        }
    }


    // let text = std::str::from_utf8(message.text().unwrap()).unwrap();
    // let fragment = Html::parse_fragment(text);
    // let selector = Selector::parse("h1").unwrap();

    // let h1 = fragment.select(&selector).next().unwrap();
    // let text = h1.text().collect::<Vec<_>>().join(" ");

    Ok(MessagePreview {title, author, time, text})
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

    let most_recent: Vec<_> = uids.iter().rev().skip(start).take(stop - start).map(|x| x.to_string()).collect();
    let messages = imap_session.fetch(most_recent.join(","), "(BODY.PEEK[HEADER] BODY.PEEK[])")?;

    messages.iter().rev().map(parse_imap_message_preview).collect()
}