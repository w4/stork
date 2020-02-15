use std::hash::{Hash, Hasher};

use futures::{pin_mut, StreamExt};

use failure::Fallible;

use stork::FilterSet;
use stork_http::{filters::*, HttpStorkable, Link};

#[derive(argh::FromArgs)]
/// Link hunter with a little bit of magic.
struct Args {
    #[argh(option)]
    /// specifies how deep we should go from the origin, leave this
    /// value unspecified to recurse until there's nothing left to
    /// follow.
    max_depth: Option<usize>,

    #[argh(switch, short = 'o')]
    /// only grab links from the same origin, useful for creating
    /// sitemaps
    same_origin: bool,

    #[argh(positional)]
    url: Link,
}

fn make_tuple_fn(
    depth: usize,
) -> impl Fn(failure::Fallible<HttpStorkable>) -> (Fallible<HttpStorkable>, usize) {
    move |v| (v, depth)
}

#[tokio::main]
async fn main() -> failure::Fallible<()> {
    let args: Args = argh::from_env();
    let url = args.url;

    let mut filters = FilterSet::default();
    if args.same_origin {
        filters = filters.add_filter(DomainFilter::new(url.url().host().unwrap().to_string()));
    }

    let queue = futures::stream::SelectAll::new();
    pin_mut!(queue);

    // push the initial Storkable onto the queue
    queue.push(Box::pin(
        HttpStorkable::new(url)
            .with_filters(filters)
            .exec()
            .map(make_tuple_fn(0)),
    ));

    let mut seen = Vec::new();

    loop {
        let value = queue.next().await;

        if value.is_none() {
            break;
        }

        let (link, depth) = value.unwrap();

        if let Err(e) = link {
            eprintln!("Failed to grab a link: {}", e);
            continue;
        }

        let link = link.unwrap();

        // TODO: see if we can do this in a filter before we even make
        // TODO: it into the synchronous print loop
        let hash = {
            let mut hash = twox_hash::XxHash64::default();
            link.val().hash(&mut hash);
            hash.finish()
        };
        if seen.contains(&hash) {
            continue;
        } else {
            seen.push(hash);
        }

        println!("{}", link.val().url());

        if let Some(max_depth) = args.max_depth {
            if depth >= max_depth {
                continue;
            }
        }

        // add children of this storkable to the front of the queue with
        // 1 depth added on
        queue.push(Box::pin(link.exec().map(make_tuple_fn(depth + 1))));
    }

    Ok(())
}
