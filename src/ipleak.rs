extern crate serde;
extern crate serde_json;
use clap::{App, Arg, ArgMatches};
use console::style;
use reqwest;
use serde::{Deserialize, Serialize};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::Table;
use term_table::TableStyle;
use toolz::core;

const IPLEAK_URL_JSON: &'static str = "https://ipleak.net/json";
// const FIELDS: [&'static str; 20] = [
//     "postal_confidence",
//     "region_code",
//     "cache",
//     "region_name",
//     "latitude",
//     "ip",
//     "accuracy_radius",
//     "time_zone",
//     "continent_code",
//     "metro_code",
//     "continent_name",
//     "city_name",
//     "longitude",
//     "country_code",
//     "query_type",
//     "country_name",
//     "postal_code",
//     "level",
//     "query_text",
//     "query_date",
// ];

#[derive(Serialize, Deserialize, Debug)]
pub struct IPLeakInfo {
    country_code: String,
    country_name: String,
    region_code: String,
    region_name: String,
    continent_code: String,
    continent_name: String,
    city_name: String,
    postal_code: Option<String>,
    postal_confidence: Option<f64>,
    latitude: f64,
    longitude: f64,
    accuracy_radius: u64,
    time_zone: String,
    metro_code: Option<String>,
    level: String,
    cache: u64,
    ip: String,
    reverse: Option<String>,
    query_text: String,
    query_type: String,
    query_date: u64,
}
impl IPLeakInfo {
    pub fn to_human_friendly_table(&self) -> String {
        let mut table = Table::new();
        table.max_column_width = 30;

        table.style = TableStyle::blank();

        table.add_row(Row::new(vec![TableCell::new_with_alignment(
            format!("Your IP address is: {}", self.ip),
            2,
            Alignment::Center,
        )]));

        table.add_row(Row::new(vec![
            TableCell::new_with_alignment("Country", 1, Alignment::Left),
            TableCell::new_with_alignment(format!("{}", self.country_name), 1, Alignment::Left),
        ]));
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment("Country Code", 1, Alignment::Left),
            TableCell::new_with_alignment(format!("{}", self.country_code), 1, Alignment::Left),
        ]));
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment("City Name", 1, Alignment::Left),
            TableCell::new_with_alignment(format!("{}", self.city_name), 1, Alignment::Left),
        ]));
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(
                format!("Latitude: {}", self.latitude),
                1,
                Alignment::Left,
            ),
            TableCell::new_with_alignment(
                format!("Longitude: {}", self.longitude),
                1,
                Alignment::Left,
            ),
        ]));
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment("Time Zone:", 1, Alignment::Left),
            TableCell::new_with_alignment(format!("{:?}", self.time_zone), 1, Alignment::Left),
        ]));

        String::from(table.render())
    }
}
#[derive(Debug)]
pub struct IPLeak {
    client: reqwest::Client,
}

static APP_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.55 Safari/537.36";

impl IPLeak {
    pub fn new<'a>() -> Result<IPLeak, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .cookie_store(true)
            .build()?;
        Ok(IPLeak { client })
    }

    pub async fn get_ip_leak_info(&self) -> IPLeakInfo {
        let data = self.request_url(IPLEAK_URL_JSON).await.unwrap();
        let info: IPLeakInfo =
            serde_json::from_str(&data.as_str()).expect("failed to parse response json");
        info
    }
    pub async fn request_url(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        // for (key, value) in response.headers(){
        //     println!("\t\t{}: {}", key, value.to_str().unwrap());
        // }
        let data = response.text().await?;
        Ok(data)
    }
}

async fn ipleak_command<'a>(matches: &'a ArgMatches<'_>) {
    let dry_run = matches.is_present("dry_run");

    if dry_run {
        return;
    }

    let client = IPLeak::new().expect("failed to create ipleak http client");
    let data = client.get_ip_leak_info().await;

    let ip_color = data.ip.split(".").collect::<Vec<&str>>()[0]
        .parse::<usize>()
        .unwrap();

    println!(
        "{}",
        style(data.to_human_friendly_table()).color256(ip_color.try_into().unwrap())
    );
    // println!(
    //     "{}",
    //     style(serde_json::to_string_pretty(&data).unwrap()).color256(ip_color.try_into().unwrap())
    // );
}

#[tokio::main]
async fn main() {
    let app = App::new("ipleak")
        .version(core::VERSION)
        .author(core::AUTHOR)
        .about("check your ip via ipleak.net")
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .short("n")
                .takes_value(false),
        );
    // for field in FIELDS {
    //     let dashed = String::from(field.replace("_", "-"));
    //     app = app.arg(Arg::with_name(field).long(&dashed).takes_value(false));
    // }

    let matches = app.get_matches();
    ipleak_command(&matches).await
}
