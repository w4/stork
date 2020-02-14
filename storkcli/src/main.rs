use futures::{pin_mut, StreamExt};
use std::collections::VecDeque;
use stork_http::HttpStorkable;

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args
        .get(1)
        .expect("Expecting URL parameter")
        .parse()
        .unwrap();

    traverse(HttpStorkable::new(url)).await?;

    Ok(())
}

async fn traverse(storkable: HttpStorkable) -> failure::Fallible<()> {
    let stream = storkable.exec();
    pin_mut!(stream); // needed for iteration

    let mut queue: VecDeque<_> = stream.map(|v| (v, 0)).collect::<VecDeque<_>>().await;

    if queue.is_empty() {
        panic!("Failed to find any links on the page!");
    }

    // TODO: this is very synchronous at the moment
    loop {
        if queue.is_empty() {
            break;
        }

        let (link, depth) = queue.pop_front().unwrap();
        let link: HttpStorkable = link?;

        println!("{}↳ {}", " ".repeat(depth), link.val().url());

        // add children of this storkable to the front of the queue with
        // 1 depth added on
        let children = link.exec();
        pin_mut!(children);

        while let Some(v) = children.next().await {
            queue.push_front((v, depth + 1));
        }

        // TODO: get the library returning futures 0.3 asap
        //        link.exec()
        //            .map(|v| (v, depth + 1))
        //            .for_each(|v| queue.push_front(v))
        //            .await;
    }

    Ok(())
}
