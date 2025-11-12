pub mod files;
pub mod style;
pub mod ui;
pub mod envlight;
pub mod objects;
pub mod thumbnails;





// async reference
/*
pub async fn read_create_dir(path: &str) -> ReadDir {
    let dir;
    match fs::read_dir(path) {
        Ok(p) => {
            dir = p;
        }

        Err(_) => {
            println!("Setting up creating {}", path);
            let res = fs::create_dir(path);
            println!("Created res {:#?}", res);
            dir = fs::read_dir(path).unwrap();
        }
    };
    dir
}


pub async fn list_dir(
    path: &str,
    //_queue_db: State<'_, QueueDb>,
    //_system_db: State<'_, SystemDb>,
) -> Result<Vec<String>, String> {
    let mut files: Vec<String> = vec![];
    //let current_dir = env::current_dir().unwrap().join(dir);

    let mut entries: Vec<FileEntry> = vec![];

    let dir = read_create_dir(path).await;

    for entry in dir {
        let entry = entry.unwrap();
        //println!("dir entry {:?}", entry.file_type());
        let file_name = entry.file_name().into_string().unwrap().to_string();
        if entry.file_type().unwrap().is_file() && !file_name.starts_with(".") {
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

            // let size = metadata.size();
            entries.push(FileEntry {
                name: path
                    //.file_name()
                    //.ok_or("No filename")
                    //.unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                last_modified,
            });
        }
    }

    entries.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));
    entries.iter().for_each(|f| files.push(f.name.to_string()));

    Ok(files)
}


pub async fn list_directories(path: &str) -> Result<Vec<String>, String> {
    let mut directories: Vec<String> = vec![];
    let mut entries: Vec<FileEntry> = vec![];
    let dir = read_create_dir(path).await;
    for entry in dir {
        let entry = entry.unwrap();
        //println!("dir entry {:?}", entry);
        let dir_name = entry.file_name().into_string().unwrap().to_string();
        // println!("dir name {:?}", dir_name);
        if entry.file_type().unwrap().is_dir() && !dir_name.starts_with(".") {
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();

            entries.push(FileEntry {
                name: path.to_str().unwrap().to_string(),
                last_modified,
            });
        }
    }

    entries.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));
    entries
        .iter()
        .for_each(|f| directories.push(f.name.to_string()));

    Ok(directories)
}
*/
