#![warn(rust_2018_idioms)]

use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::RcDom;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn get_dom(page_content: String) -> RcDom {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut page_content.as_bytes())
        .unwrap()
}

pub async fn fetch_url_body(url: &String) -> Result<String> {
    let mut res = surf::get(url).await?;
    let body = res.body_string().await?;
    //    println!("\n**********************START PAGE BODY Body*******************\n\n{}\n\n**********************END PAGE BODY**********************\n", body);
    Ok(body)
}
