use futures::{pin_mut, StreamExt};
use std::collections::VecDeque;
use stork_http::{HttpStorkable, Link};

#[derive(argh::FromArgs)]
/// Link hunter with a little bit of magic.
struct Args {
    #[argh(option)]
    /// specifies how deep we should go from the origin, leave this
    /// value unspecified to recurse until there's nothing left to
    /// follow.
    max_depth: Option<usize>,

    #[argh(positional)]
    url: Link,
}

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let args: Args = argh::from_env();
    let url = args.url;

    let stream = HttpStorkable::new(url).exec();
    pin_mut!(stream); // needed for iteration

    let mut queue = stream.map(|v| (v, 0)).collect::<VecDeque<_>>().await;

    if queue.is_empty() {
        panic!("Failed to find any links on the page!");
    }

    // TODO: this is very synchronous at the moment
    loop {
        if queue.is_empty() {
            break;
        }

        let (link, depth) = queue.pop_front().unwrap();

        if let Err(e) = link {
            eprintln!("Failed to grab a link: {}", e);
            continue;
        }

        let link = link.unwrap();

        println!("{}↳ {}", " ".repeat(depth), link.val().url());

        if let Some(max_depth) = args.max_depth {
            if depth >= max_depth {
                continue;
            }
        }

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
