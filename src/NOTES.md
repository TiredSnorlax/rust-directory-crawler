# Steps
1. Get a directory
2. Loop through directory -> process files, add child directories to a queue
3. 

CrawlResult {
  files_found: u64,
  file_types: HashSet<String>
  directories: Vec<PathBuf>
}



## Main func:
1. Get the main directory and execute it in a thread
2. Main Loop: 
      1. Wait to recieve CrawlResults
        - Remove current directory from directories being crawled
      2. If CrawlResults returns no directories && no other worker is working -> break
        - Keep track of the directories being crawled in a vec
      3. Add CrawlResults to tracker
      4. Execute crawl on directories returned
        - Add directory to directories being crawled
