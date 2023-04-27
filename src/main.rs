use colored::*;
use soup::prelude::*;
use std::io;
use terminal_menu::{button, label, menu, mut_menu, run};

enum FilterByDate {
    LastHour,
    Last24Hours,
    Yesterday,
    LastWeek,
    LastMonth,
    LastYear,
    All,
}

struct New {
    title: String,
    link: Option<String>,
    datetime: String,
    datetime_formated: String,
}

async fn get_news(
    query: &str,
    filter_by_date: FilterByDate,
    language: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let period = match filter_by_date {
        FilterByDate::LastHour => "+when:1h",
        FilterByDate::Last24Hours => "+when:1d",
        FilterByDate::Yesterday => "+when:2d",
        FilterByDate::LastWeek => "+when:7d",
        FilterByDate::LastMonth => "+when:1m",
        FilterByDate::LastYear => "+when:1y",
        FilterByDate::All => "",
    };

    let url = format!(
        "https://news.google.com/search?q={}{}&hl={}",
        query, period, language
    );
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

fn parse_news(news: String, show_link: bool) -> Vec<New> {
    let soup = Soup::new(&news);
    let articles = soup.tag("article").find_all();

    let mut news: Vec<New> = Vec::new();
    for article in articles {
        let title = article.tag("h3").find().unwrap().text();

        if title.len() == 0 {
            continue;
        }

        let link = article.tag("a").find().unwrap().get("href").unwrap();
        let link = &link[1..];

        let datetime = article.tag("time").find().unwrap().get("datetime").unwrap();
        let date_formated = article.tag("time").find().unwrap().text();

        let new = New {
            title: title,
            link: if show_link {
                Some(format!("https://news.google.com{}", link))
            } else {
                None
            },
            datetime: datetime,
            datetime_formated: date_formated,
        };

        news.push(new);
    }

    news
}

fn print_new(new: &New, date_formated: bool, space: usize) {
    let title = format!("{}", new.title).bold().purple();
    let datetime = if date_formated {
        format!("{}", new.datetime_formated).bold().blue()
    } else {
        format!("{}", new.datetime).bold().blue()
    };

    if datetime.len() <= space {
        // add spaces to the end
        let spaces = space - datetime.len();
        let mut spaces_str = String::new();
        for _ in 0..spaces {
            spaces_str.push(' ');
        }
        print!("{}{} {}", datetime, spaces_str, title);
    }

    if let Some(link) = &new.link {
        println!("\n{}", link);
    }

    println!();
}

#[tokio::main]
async fn main() {
    let mut user_input = String::new();
    println!("Digite o que deseja buscar: ");
    io::stdin()
        .read_line(&mut user_input)
        .expect("Erro ao ler entrada do usuário");

    let query = user_input.trim();

    let menu = menu(vec![
        label("Escolha um período:"),
        button("Última hora"),
        button("Últimas 24 horas"),
        button("Ontem"),
        button("Última semana"),
        button("Último mês"),
        button("Último ano"),
        button("Tudo"),
    ]);
    run(&menu);
    let choice = mut_menu(&menu).selected_item_name().to_string();
    let show_link = false;

    let filter_by_date = match choice.as_str() {
        "Última hora" => FilterByDate::LastHour,
        "Últimas 24 horas" => FilterByDate::Last24Hours,
        "Ontem" => FilterByDate::Yesterday,
        "Última semana" => FilterByDate::LastWeek,
        "Último mês" => FilterByDate::LastMonth,
        "Último ano" => FilterByDate::LastYear,
        "Tudo" => FilterByDate::All,
        _ => FilterByDate::All,
    };

    let news = get_news(query, filter_by_date, "ptbr").await.unwrap();

    if news.len() == 0 {
        println!("Nenhuma notícia encontrada");
        return;
    }
    let news = parse_news(news, show_link);

    let mut biggest_date = 0;
    for new in &news {
        if new.title.len() > biggest_date {
            biggest_date = new.datetime_formated.len();
        }
    }
    let date_formated = true;
    for new in &news {
        print_new(&new, date_formated, biggest_date);
    }
}
