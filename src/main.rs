use crate::crawler::Crawler;
use std::env;

mod crawler;
mod pool;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Enter a directory name");
        return;
    }

    let path = &args[1];

    let mut crawler = Crawler::default();

    let res = crawler.crawl_directory(path.clone());
    match res {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e)
        }
    }
}
