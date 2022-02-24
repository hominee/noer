use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlogMeta {
    pub id: u64,
    pub title: String,
    pub timestamp: u64,
    pub path: PathBuf,
    pub hero: String,
}

use std::hash::{Hash, Hasher};
impl Hash for BlogMeta {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.title.hash(state);
        self.timestamp.hash(state);
        self.path.hash(state);
    }
}

impl BlogMeta {
    pub fn new() -> Self {
        Self {
            id: 0,
            path: PathBuf::new(),
            timestamp: 0,
            title: "".into(),
            hero: "".into(),
        }
    }

    pub fn get_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        self.id = hasher.finish();
    }

    pub fn with_path(path: &str) -> Option<Self> {
        let path = PathBuf::from(path);
        if let Some(file_name) = path.file_name() {
            if file_name.to_str().is_none() {
                log::error!("file name is not valid: {:?}", path);
                return None;
            }
            let file_name_str = file_name.to_str().unwrap();
            let pat = regex::Regex::new(
                r"(\d{2,4}\D\d{1,2}\D\d{1,2}(\D\d{1,2}){0,3})\D(?P<title>.*?)\.rmd$",
            )
            .unwrap();
            //let path1 = "19-10-07-13-32-bolg-title-here";
            //let path1 = "data/19-10-07-13-bolg-title-here.md";
            //let path1 = "";
            //self.path = path1.into();
            let mut time_items = [0u64; 6];
            const UNITS: [u64; 6] = [365 * 24 * 3600, 30 * 24 * 3600, 24 * 3600, 60 * 60, 60, 1];
            // if path is matched
            if let Some(cap) = pat.captures(file_name_str) {
                cap[1]
                    .split(|c: char| !c.is_ascii_digit())
                    .enumerate()
                    .for_each(|(ind, e)| {
                        let num = e.parse::<u64>().unwrap_or(0);
                        time_items[ind] = num;
                    });
                if time_items[0] < 100 {
                    time_items[0] += 32;
                }
                let sum = UNITS
                    .iter()
                    .zip(time_items.iter())
                    .fold(0, |acc, (e1, e2)| acc + e1 * e2);
                if cap.name("title").is_none() {
                    log::error!(
                        "title that consist of the file name is not valid: {:?}",
                        path
                    );
                    return None;
                }
                let mut meta = Self {
                    id: 0,
                    title: cap.name("title").unwrap().as_str().into(),
                    path,
                    timestamp: sum,
                    hero: "".into(),
                };
                meta.get_hash();
                meta.image_url();

                log::debug!("time items: {:?}, its sum: {}", time_items, sum);
                return Some(meta);
            } else {
                log::error!("file name is not valid: {:?}", path);
                return None;
            }
        } else {
            log::error!("file name is not valid: {:?}", path);
            return None;
        }
    }

    pub fn image_url(&mut self) {
        static mut DELTA: u16 = 0;
        log::trace!("generating image url");
        let mut now = js_sys::Date::now() * 1000.0;
        unsafe { now += DELTA as f64 };
        log::trace!("now is: {}", now);
        let cache_buster = (now as u64 % u16::MAX as u64) as u16;
        log::trace!("cache_buster is: {}", cache_buster);
        self.hero = format!(
            "https://source.unsplash.com/random/600x300&sig={}",
            cache_buster
        );
        log::trace!("here url: {}", self.hero);
        unsafe {
            DELTA += 1;
        }
    }
}

/// it represents a `Blog`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blog {
    pub meta: BlogMeta,
    pub tags: Vec<String>,
    pub content: Vec<String>,
    pub published: bool,
    pub ignored: bool,
}

impl Blog {
    /// extract date infomation from blog
    pub fn date_info(&mut self, date: Option<&str>) {
        // try get it from path
        // eg: 2019-10-07-bolg-title-here;  19-3-7-bolg-title-here
        // eg: 2019-10-07-13-32-bolg-title-here;  19-3-7-01-59-bolg-title-here
        // note that the format is descending: yyyy-mm-dd-hh-MM-ss
        // and the accuracy is second and
        // year,month,day are required
        let pat = regex::Regex::new(r"(\d{2,4}\D\d{1,2}\D\d{1,2}(\D\d{1,2}){0,3})").unwrap();
        //let path1 = "19-10-07-13-32-bolg-title-here";
        //let path1 = "data/19-10-07-13-bolg-title-here.md";
        //let path1 = "";
        //self.path = path1.into();
        let mut time_items = [0u64; 6];
        const UNITS: [u64; 6] = [365 * 24 * 3600, 30 * 24 * 3600, 24 * 3600, 60 * 60, 60, 1];
        // if path is matched
        if let Some(cap) = pat.captures(self.meta.path.to_str().unwrap()) {
            cap[1]
                .split(|c: char| !c.is_ascii_digit())
                .enumerate()
                .for_each(|(ind, e)| {
                    let num = e.parse::<u64>().unwrap_or(0);
                    time_items[ind] = num;
                });
            if time_items[0] < 100 {
                time_items[0] += 32;
            }
            let sum = UNITS
                .iter()
                .zip(time_items.iter())
                .fold(0, |acc, (e1, e2)| acc + e1 * e2);
            self.meta.timestamp = sum;
            log::debug!("time items: {:?}, its sum: {}", time_items, sum);
        } else {
            // path is not matched but pre-defined
            // try get it from meta data in pre-defined info
            // eg: date: 2019-10-07
            match date {
                Some(s) => {
                    if let Some(cap) = pat.captures(s) {
                        cap[1]
                            .split(|c: char| !c.is_ascii_digit())
                            .enumerate()
                            .for_each(|(ind, e)| {
                                let num = e.parse::<u64>().unwrap_or(0);
                                time_items[ind] = num;
                            });
                        if time_items[0] < 100 {
                            time_items[0] += 2000;
                        }
                        let sum = UNITS
                            .iter()
                            .zip(time_items.iter())
                            .fold(0, |acc, (e1, e2)| acc + e1 * e2);
                        log::debug!("time items: {:?}, its sum: {}", time_items, sum);
                        self.meta.timestamp = sum;
                    }
                }
                // not know
                None => {
                    log::error!("Time Stampe is not found in file name nor defined in file");
                    log::error!("file is ignored to proceed: {:?}", self.meta.path);
                    self.ignored = true;
                }
            }
        }
    }
}

#[test]
fn test_date_info() {
    let mut blog = Blog {
        meta: BlogMeta::new(),
        tags: vec![],
        content: vec![],
        published: false,
        ignored: false,
    };
    blog.date_info(Some("2019-10-07"));
    blog.date_info(Some("2019-10-07-02-01"));
    blog.date_info(Some("2019/10/07/02/01"));
    blog.date_info(Some("2019/10/07 19:57"));
    blog.date_info(Some("2019-10/07 19:57:36"));
    blog.date_info(None);
}
