#![allow(dead_code)]
//! Here provide a module that accepts a markdown file to
//! reproduce a struct `Post`
//! make it static based blog
use crate::BlogMeta;
use crate::{constant::ITEMS_PER_PAGE, content::Blog, pages::post::read_file};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew::prelude::*;

/// read the index of directory
pub async fn read_dir() -> Vec<String> {
    use crate::constant;
    use wasm_bindgen_futures::JsFuture;
    log::trace!("{}", &format!("read dir",));
    let dir = if constant::SUBPATH != "/" {
        format!("{}{}", constant::SUBPATH, "posts/")
    } else {
        "posts/".into()
    };
    log::debug!("{}", &format!("parse dir: {:?}", dir));
    let window = web_sys::window().unwrap();
    let protocol = window
        .location()
        .protocol()
        .expect("Protocol is NULL")
        .to_string();
    let host = window
        .location()
        .host()
        .expect("HOST is not NULL")
        .to_string();
    let mut url = format!("{}//{}/{}", protocol, host, dir);
    if !url.ends_with("/") {
        url.push_str("/");
    }
    url.push_str("markdown.index");
    log::debug!("{}", &format!("url: {:?}", url));
    let res = JsFuture::from(window.fetch_with_str(&url)).await.unwrap();
    assert!(res.is_instance_of::<web_sys::Response>());
    let res_val = res.dyn_into::<web_sys::Response>().unwrap();
    let text = JsFuture::from(res_val.text().unwrap())
        .await
        .unwrap()
        .as_string()
        .expect("the index shall return string");
    log::trace!("{}", &format!("content {:?}", text));
    text.split("\n")
        .map(|e| e.trim().to_string())
        .filter(|e| e.len() != 0 && e.ends_with("rmd"))
        .collect()
}

pub fn str2blog(s: &str, meta: &BlogMeta) -> Option<Blog> {
    log::trace!("parsing a string into a blog");
    let sp = s.splitn(3, "---").collect::<Vec<_>>();
    let metadata = sp[1].trim();
    let cont = sp[2]
        .splitn(2, "<!--break-->")
        .map(|e| e.trim().to_owned())
        .collect::<Vec<String>>();
    let pat = regex::Regex::new(r"(?P<key>\w+)\s*:\s*(?P<value>.*?$)").unwrap();
    let pat_tag = regex::Regex::new(r"\s*-\s+(?P<tag>.*?$)").unwrap();
    let pat_ml = regex::Regex::new(r"(\s{2,}||\t{1,})(?P<ln>.*?$)").unwrap();
    let mut map = HashMap::new();
    let mut tags = Vec::new();
    let mut lastkey = "";
    for line in metadata.split("\n") {
        if line.trim().len() < 1 {
            // empty line skip it
            lastkey = "";
            continue;
        }
        if let Some(caps) = pat.captures(line) {
            if caps.len() == 1 {
                log::trace!("Not Matched: {}", line);
                break;
            }
            let key = caps.name("key").map_or("", |e| e.as_str());
            lastkey = key;
            let value = caps.name("value").map_or("", |e| e.as_str());
            if key == "tags" {
                lastkey = key;
            } else {
                map.insert(key, value.trim().to_owned());
            }
            continue;
        }
        if let Some(caps) = pat_tag.captures(line) {
            if let Some(tag) = caps.name("tag") {
                let tag = tag.as_str().trim().to_owned();
                tags.push(tag);
                continue;
            }
        }
        if let Some(caps) = pat_ml.captures(line) {
            let ln = caps.name("ln").unwrap().as_str().trim();
            if let Some(item) = map.get_mut(&lastkey) {
                item.push_str(ln);
            } else {
                log::debug!("Line ingnored: {}", ln);
            }
        }
    }
    if map.get(&"title").is_none() {
        log::error!("Title Missing for {:?}", meta.path);
        return None;
    }
    if map.get(&"published").is_none() {
        log::error!("Attribute `published` Missing for {:?}", meta.path);
        return None;
    }
    let mut blog = Blog {
        meta: meta.clone(),
        tags,
        content: cont,
        published: map
            .remove("published")
            .unwrap_or("false".into())
            .parse::<bool>()
            .unwrap(),
        ignored: false,
    };
    match map.remove("date") {
        Some(s) => blog.date_info(Some(&s)),
        None => blog.date_info(None),
    }
    blog.meta.get_hash();
    println!("blog path: {:?}", blog.meta.path);
    Some(blog)
}

/// the order of post sort
#[derive(PartialEq, Clone, Debug)]
pub enum Order {
    Dec,
    Inc,
    Hot,
}

impl Default for Order {
    fn default() -> Self {
        Self::Dec
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseAct {
    CacheBlog(Blog),
    MoreBlogMeta,
    BlogPath(Vec<String>),
    ChangeDisplayMode,
}

/// the inner data of Parser
#[derive(Clone, Default, Debug, PartialEq)]
pub struct InnerParser {
    pub blogs: HashMap<u64, Blog>,
    pub indexs: Vec<(u64, u64)>,
    pub records: HashMap<u64, u8>,
    pub len: usize,
}
impl InnerParser {
    pub fn inc(&mut self, id: &u64, inc: u8) {
        if self.records.get(id).is_some() {
            *self.records.get_mut(id).unwrap() += inc;
        } else {
            self.records.insert(*id, inc);
        }
    }
}

/// it represents a `Parser` that parse all markdown file of directory
#[derive(Clone, Default, Debug, PartialEq, Properties)]
pub struct Parser {
    pub metas: HashMap<u64, BlogMeta>,
    pub ids: Vec<u64>,
    pub paths: Vec<String>,
    pub offset: usize,
    inner: InnerParser,
    pub order: Order,
    pub parsed: bool,
    pub display: String,
}

impl Reducible for Parser {
    type Action = ParseAct;
    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ParseAct::CacheBlog(blog) => (*Rc::make_mut(&mut self)).insert(blog.meta.id, blog),
            ParseAct::MoreBlogMeta => (*Rc::make_mut(&mut self)).load_meta(ITEMS_PER_PAGE),
            ParseAct::BlogPath(paths) => (Rc::make_mut(&mut self)).paths = paths,
            ParseAct::ChangeDisplayMode => match &self.display as &str {
                "gridCard" => (Rc::make_mut(&mut self)).display = "listTile".into(),
                "listTile" => (Rc::make_mut(&mut self)).display = "gridCard".into(),
                _ => {}
            },
        }
        self
    }
}

/// implementation of Parser to load markdown file as Blog
impl Parser {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            metas: HashMap::new(),
            paths: Vec::new(),
            offset: 0,
            inner: InnerParser {
                blogs: HashMap::new(),
                indexs: vec![],
                records: HashMap::new(),
                len: 0,
            },
            order: Order::Dec,
            parsed: false,
            display: "listTile".into(),
        }
    }

    pub fn change_ord(&mut self, ord: Order) {
        if ord == self.order {
            // nothing to change
            log::debug!("ord not changed");
        } else {
            // change the order of blogs
            self.order = ord;
            self.order();
        }
    }

    ///sort the blog with the Order
    pub fn order(&mut self) {
        let cmp_fn = match self.order {
            Order::Inc => |a: &(u64, u64), b: &(u64, u64)| -> std::cmp::Ordering {
                a.0.partial_cmp(&b.0).unwrap()
            },
            Order::Dec => |a: &(u64, u64), b: &(u64, u64)| -> std::cmp::Ordering {
                b.0.partial_cmp(&a.0).unwrap()
            },
            Order::Hot => |a: &(u64, u64), b: &(u64, u64)| -> std::cmp::Ordering {
                b.0.partial_cmp(&a.0).unwrap()
            },
        };
        self.inner.indexs.sort_by(cmp_fn);
    }

    /// get the length of the blogs
    pub fn len(&self) -> usize {
        self.inner.len
    }

    pub fn all_page(&self) -> u64 {
        let len = self.paths.len() as u64;
        let num = len / ITEMS_PER_PAGE;
        let remainder = len - ITEMS_PER_PAGE * num;
        match remainder as u64 {
            0 => num,
            _ => num + 1,
        }
    }

    /// all availible paths that can made into pages
    pub fn total_page(&self) -> u64 {
        let len = self.ids.len() as u64;
        let num = len / ITEMS_PER_PAGE;
        let remainder = len - ITEMS_PER_PAGE * num;
        match remainder as u64 {
            0 => num,
            _ => num + 1,
        }
    }

    /// get BlogMeta by path
    pub fn get_meta(&self, id: &u64) -> Option<&BlogMeta> {
        self.metas.get(id)
    }

    pub fn load_meta(&mut self, cnt: u64) {
        let mut delta = 0;
        for index in 0..cnt {
            let offset = self.offset as usize + index as usize;
            if offset >= self.paths.len() {
                break;
            }
            let path = &self.paths[offset];
            if let Some(item) = BlogMeta::with_path(path) {
                self.ids.push(item.id);
                self.metas.insert(item.id, item);
            }
            delta += 1;
        }
        self.offset += delta;
        if !self.parsed {
            self.parsed = true;
        }
    }

    /// insert blog into inner
    pub fn insert(&mut self, key: u64, val: Blog) {
        self.inner.indexs.push((key, val.meta.timestamp));
        self.inner.len += 1;
        self.inner.inc(&key, 1);
        self.inner.blogs.insert(key, val);
    }

    pub fn inner(&self) -> &InnerParser {
        &self.inner
    }

    /// get the blog with the index
    pub fn get(&self, index: usize) -> Option<&Blog> {
        match self.inner.indexs.get(index) {
            Some(path) => self.inner.blogs.get(&path.1),
            None => None,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.inner)
    }

    pub async fn load(&mut self, batch_size: u64) {
        let mut cnt = 0;
        if self.metas.len() > self.offset {
            for ind in 0..batch_size as usize {
                let id = self.ids[ind + self.offset];
                let meta = &self.metas.get(&id).unwrap();
                log::debug!("path: {:?}", meta.path);
                self.inner.inc(&meta.id, 1);
                if let Ok(buf) = read_file(meta.path.to_str().unwrap()).await {
                    cnt += 1;
                    if !buf.is_empty() {
                        // parse the string to Blog
                        let blog = str2blog(&buf, meta);
                        if let Some(Blog { ignored: false, .. }) = blog {
                            let blog = blog.unwrap();
                            self.inner.indexs.push((blog.meta.timestamp, meta.id));
                            self.inner.blogs.insert(meta.id, blog);
                            self.inner.len += 1;
                            assert_eq!(self.inner.indexs.len(), self.inner.len);
                            assert_eq!(self.inner.blogs.len(), self.inner.len);
                        }
                    } else {
                        // empty file
                        log::info!("Empty File: {:?}", meta.path);
                    }
                }
            }
            self.offset += cnt;
        }
        self.order();
    }
}

pub struct Iter<'a> {
    data: &'a HashMap<u64, Blog>,
    index: &'a Vec<(u64, u64)>,
    offset: usize,
    len: usize,
}
impl<'a> Iter<'a> {
    pub fn new(data: &'a InnerParser) -> Self {
        Self {
            data: &data.blogs,
            index: &data.indexs,
            offset: 0,
            len: data.indexs.len(),
        }
    }
}

unsafe impl Sync for Iter<'_> {}
unsafe impl Send for Iter<'_> {}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Blog;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.len {
            let (_, path) = self.index.get(self.offset).unwrap();
            self.offset += 1;
            return Some(self.data.get(path).unwrap());
        }
        None
    }
}
