use std::{
    collections::HashSet,
    fs::read_dir,
    io::Error,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Instant,
};

use crate::pool::ThreadPool;

#[derive(Default)]
pub struct DirectoryCrawl {
    directory: PathBuf,
    files_found: u64,
    file_types: HashSet<String>,
    directories: Vec<PathBuf>,
    size: u64,
}

#[derive(Default, Debug)]
struct CrawlResults {
    files_found: u64,
    directories_searched: u64,
    file_types: HashSet<String>,
    total_size: u64,
}

#[derive(Default)]
pub struct Crawler {
    directories_being_crawled: u64,
}

impl Crawler {
    pub fn crawl_directory(&mut self, dir: String) -> Result<(), Error> {
        let start = Instant::now();
        let (res_tx, res_rx) = mpsc::channel();
        // Using this somehow slows the program down.
        // Not sure why
        // let num_workers = thread::available_parallelism().unwrap();
        // 6 is the ideal number for my PC
        let num_workers = 6;
        println!("Spawning {num_workers} workers");
        let thread_pool = ThreadPool::new(num_workers, res_tx);
        let mut crawl_results = CrawlResults::default();

        let main_dir_path = Path::new(&dir).to_path_buf();

        self.directories_being_crawled += 1;
        thread_pool.execute(|| Self::read_dir(main_dir_path));

        loop {
            let msg = res_rx.recv().unwrap();
            match msg {
                Ok(dir_crawl) => {
                    self.directories_being_crawled -= 1;
                    crawl_results.file_types.extend(dir_crawl.file_types);
                    crawl_results.files_found += dir_crawl.files_found;
                    crawl_results.total_size += dir_crawl.size;

                    for directory in dir_crawl.directories {
                        crawl_results.directories_searched += 1;
                        self.directories_being_crawled += 1;
                        thread_pool.execute(|| Self::read_dir(directory));
                    }
                }
                Err(e) => {
                    println!("{e:?}");
                    self.directories_being_crawled -= 1;
                    break;
                }
            }
            if self.directories_being_crawled == 0 {
                break;
            }
        }

        println!("{:?}", crawl_results);
        let duration = start.elapsed();
        println!("Took {:?} long", duration);
        Ok(())
    }

    fn read_dir(path: PathBuf) -> Result<DirectoryCrawl, Error> {
        // self.directories_being_crawled.insert(path.clone());
        let mut directory_crawl = DirectoryCrawl::default();
        directory_crawl.directory = path.clone();
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = path.metadata()?;

            // We use metadata.is_file() here instead of path.is_file() for 2 reasons
            // 1. We need metadata info later anyways
            // 2. Both path.metadata() and path.is_file() are expensive system calls.
            //  -> Calling is_file() on the metadata instead, will reduce the number of system calls to just path.metadata()
            if metadata.is_file() {
                directory_crawl.files_found += 1;
                if let Some(file_type) = path.extension() {
                    directory_crawl.size += metadata.len();
                    let file_type = file_type.to_string_lossy().into_owned();
                    directory_crawl.file_types.insert(file_type);
                }
            } else if metadata.is_dir() {
                directory_crawl.directories.push(path);
            }
        }
        Ok(directory_crawl)
    }
}
