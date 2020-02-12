use futures::pin_mut;
use futures::stream::StreamExt;
use stork::filters::{UrlFilter, UrlFilterType};

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(1).expect("Expecting URL parameter").parse().unwrap();

    let stream = stork::Storkable::new(url)
//        .with_filters(
//            stork::Filters::default()
//                .add_url_filter(UrlFilter::new(
//                    UrlFilterType::Domain,
//                    "stackoverflow.blog".to_string()))
//        )
        .exec();
    pin_mut!(stream); // needed for iteration

    while let Some(link) = stream.next().await {
        if let Err(err) = link {
            eprintln!("{:#?}", err);
            continue;
        }
        let link = link.unwrap();

        println!("{}", link.url());

        let stream = link.exec();
        pin_mut!(stream); // needed for iteration

        while let Some(link) = stream.next().await {
            if let Err(err) = link {
                eprintln!("{:#?}", err);
                continue;
            }
            println!("> {}", link.unwrap().url());
        }
    }

    Ok(())
}