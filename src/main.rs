use anyhow::{format_err, Error};
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use std::{
    fmt::{self, Write},
    fs, io,
};

fn main() -> Result<(), Error> {
    lazy_static! {
        static ref SECTION: Selector = Selector::parse("section#contentCols").unwrap();
        static ref DL: Selector = Selector::parse("dl").unwrap();
    }
    let doc_raw = load_html()?;
    let doc = Html::parse_document(&doc_raw);
    let section = doc.select(&SECTION).next().unwrap();
    //println!("{}", section.html());
    let categories = section
        .select(&DL)
        .map(Category::from_html)
        .filter(|category| match category {
            Ok(c) => c.title != "Various other",
            _ => true,
        })
        .collect::<Result<Vec<Category>, Error>>()?;
    for category in categories {
        println!("{}", category);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Category {
    /// The name of the category if it has one
    pub title: String,
    /// The APIs in the category
    pub items: Vec<Api>,
}

#[derive(Debug)]
pub struct Api {
    /// The name of the API
    name: String,
    /// A link to the API specification
    spec: String,
}

impl Category {
    pub fn from_html(el: ElementRef<'_>) -> Result<Self, Error> {
        Ok(Self {
            title: Self::title(el)?,
            items: Self::items(el)?,
        })
    }

    fn title(el: ElementRef<'_>) -> Result<String, Error> {
        lazy_static! {
            static ref DT: Selector = Selector::parse("dt").unwrap();
        }

        Ok(join_text(el.select(&DT).next().ok_or(format_err!(
            "cannot find title element in category"
        ))?))
    }

    fn items(el: ElementRef<'_>) -> Result<Vec<Api>, Error> {
        lazy_static! {
            static ref DD: Selector = Selector::parse("dd").unwrap();
        }
        el.select(&DD).map(Api::from_html).collect()
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "# {}\n", self.title)?;
        for api in self.items.iter() {
            write!(f, "  - {}\n", api)?;
        }
        Ok(())
    }
}

impl Api {
    fn from_html(el: ElementRef<'_>) -> Result<Self, Error> {
        lazy_static! {
            static ref A: Selector = Selector::parse("a").unwrap();
        }
        let mut a_iter = el.select(&A);
        let name = a_iter
            .next()
            .ok_or(format_err!("no `a` element in an api element"))?;
        Ok(Api {
            name: join_text(name),
            spec: name
                .value()
                .attr("href")
                .ok_or(format_err!("no `href` on api element title `a`"))?
                .to_owned(),
        })
    }
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]({})", self.name, self.spec)
    }
}

fn join_text(el: ElementRef<'_>) -> String {
    let mut iter = el.text();
    let first = match iter.next() {
        Some(v) => v.to_owned(),
        None => return String::new(),
    };
    iter.fold(first, |mut s, itm| {
        write!(s, " {}", itm).unwrap();
        s
    })
}

/// cache the html locally and load it
fn load_html() -> Result<String, Error> {
    const HTML_LOC: &'static str = "/tmp/web_api_scraper.html";
    match fs::read_to_string(HTML_LOC) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let html = fetch_html()?;
            fs::write(HTML_LOC, &html)?;
            Ok(html)
        }
        Err(e) => Err(e.into()),
    }
}

fn fetch_html() -> Result<String, Error> {
    Ok(reqwest::blocking::get("https://platform.html5.org/")?.text()?)
}
