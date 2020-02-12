use futures::pin_mut;
use futures::stream::StreamExt;
use stork::filters::{UrlFilter, UrlFilterType};

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(1).expect("Expecting URL parameter").parse().unwrap();

    let stream = stork::Storkable::new(url).exec();
    pin_mut!(stream); // needed for iteration

    while let Some(link) = stream.next().await {
        let link = link?;

        println!("{}", link.url());

        let stream = link.exec();
        pin_mut!(stream); // needed for iteration

        while let Some(link) = stream.next().await {
            println!("> {}", link?.url());
        }
    }

    Ok(())
}