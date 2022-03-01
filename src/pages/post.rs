use crate::parser::ParseAct;
use crate::ParseActContext;
use crate::{parser::str2blog, Blog};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::{html, prelude::*, Component, Context, Html};

/// read the file
pub async fn read_file(path: &str) -> Result<String, FetchError> {
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
    let mut url = format!("{}//{}/", protocol, host,);
    use crate::constant;
    if constant::SUBPATH != "/" {
        url.push_str(constant::SUBPATH);
    }
    url.push_str(path);
    log::debug!("{}", &format!("url: {:?}", url));
    let res = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url)).await?;
    assert!(res.is_instance_of::<web_sys::Response>());
    let res_val = res.dyn_into::<web_sys::Response>()?;
    let response_text = wasm_bindgen_futures::JsFuture::from(res_val.text()?).await?;
    let ret = response_text.as_string().unwrap();

    log::trace!("path: {:?} \n content: {:?}", url, ret);
    return Ok(ret);
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
    pub title: String,
}

/// Something wrong has occurred while fetching an external resource.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    pub err: JsValue,
}
impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(FetchError),
}

#[allow(dead_code)]
pub enum Msg {
    SetMarkdownFetchState(FetchState<Blog>),
    GetMarkdown,
    Cached,
    Notified,
}

pub struct Post {
    post: FetchState<Blog>,
    notified: bool,
}
impl Component for Post {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (parser, _) = ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        let id = ctx.props().id;
        let item = parser.inner().blogs.get(&id);
        if item.is_some() {
            log::debug!("Cached: {}", ctx.props().title);
            return Self {
                post: FetchState::Success(item.unwrap().clone()),
                notified: false,
            };
        }
        let cnt = parser.inner().records.get(&id);
        if matches!(cnt, Some(x) if *x > 3) {
            log::debug!("Failed: {}", ctx.props().title);
            return Self {
                post: FetchState::Failed(FetchError {
                    err: JsValue::from("Failed more than 3 times"),
                }),
                notified: false,
            };
        }
        log::debug!("Not Fetcded: {}", ctx.props().title);
        Self {
            post: FetchState::NotFetching,
            notified: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::trace!("updating");
        match msg {
            Msg::Notified => {
                self.notified = true;
                true
            }
            Msg::Cached => false,
            Msg::SetMarkdownFetchState(state) => {
                log::trace!("set state");
                self.post = state;
                true
            }
            Msg::GetMarkdown => {
                log::trace!("get markdown");
                let (parser, _) = ctx
                    .link()
                    .context::<ParseActContext>(Callback::noop())
                    .expect("Parser Context not found");
                let meta = parser.get_meta(&ctx.props().id);
                if meta.is_none() {
                    ctx.link()
                        .send_message(Msg::SetMarkdownFetchState(FetchState::Failed(FetchError {
                            err: "Blog Not Found".into(),
                        })));
                    return false;
                }
                let meta = meta.unwrap().clone();
                ctx.link().send_future(async move {
                    match read_file(meta.path.to_str().unwrap()).await {
                        Ok(md) => {
                            let data = str2blog(&md, &meta).unwrap();
                            parser.dispatch(ParseAct::CacheBlog(data.clone()));
                            log::trace!("async load {}, len: {}", data.meta.title, parser.len());
                            Msg::SetMarkdownFetchState(FetchState::Success(data))
                        }

                        Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed(err)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let onclick = _ctx.link().callback(|_| Msg::Notified);
        match &self.post {
            FetchState::NotFetching => {
                _ctx.link().send_message(Msg::GetMarkdown);
                {
                    if !self.notified {
                        html! {
                           <div class="container mt-24 notification is-light">
                               <button class="delete" {onclick}> </button>
                               <br />
                               <strong class="is-large">{ " Blogs Not Fetched" }</strong>
                               <br />
                           </div>
                        }
                    } else {
                        html! {}
                    }
                }
            }
            FetchState::Fetching => html! {},
            FetchState::Success(blog) => {
                let keywords = blog
                    .tags
                    .iter()
                    .map(|tag| html! { <span class="tag is-info">{ tag }</span> });
                html! {
                    <>
                        <section class="hero is-medium is-light has-background">
                            <img alt="The post's background" class="hero-background is-transparent" src={blog.meta.hero.clone()} />
                            <div class="hero-body">
                                <div class="content">
                                    <h1 class="title">
                                        { &blog.meta.title.replace("-", " ") }
                                    </h1>
                                    <div class="tags">
                                        { for keywords }
                                    </div>
                                </div>
                            </div>
                        </section>
                        <div class="section container">
                            { self.view_content() }
                        </div>
                    </>
                }
            }
            FetchState::Failed(err) => html! {
                { if !self.notified {
                   html!{
                    <div class="container mt-24 notification is-danger">
                        <button class="delete" {onclick}> </button>
                    <strong class="is-large">{ " Blog Not Loaded Due To:" }</strong>
                    <br />
                   <p class="center"> { err.err.as_string().unwrap_or("Unexpected Error".into()) }</p>
                    </div>
                   } } else { html!{} }
                }
            },
        }
    }
}
impl Post {
    fn view_content(&self) -> Html {
        let dom_parser = web_sys::DomParser::new().unwrap();
        let mut parts = Vec::new();
        if let FetchState::Success(ref blog) = self.post {
            use pulldown_cmark::Options;
            let mut options = Options::empty();
            options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);
            blog.content.iter().for_each(|part| {
                let parser = pulldown_cmark::Parser::new_ext(&part, options);
                let mut output = String::with_capacity(part.len() * 3 / 2);
                pulldown_cmark::html::push_html(&mut output, parser);
                let output_div = format!(
                    "<div class = \"markdown-body\"> {} </div>",
                    output //.replace("<p>", "\n").replace("</p>", "<br/>")
                );
                log::debug!("{}", &format!("parsing markdown into html"));
                if let Ok(element) =
                    dom_parser.parse_from_string(&output_div, web_sys::SupportedType::TextHtml)
                {
                    log::debug!("{}", &format!("Done: parsing markdown into html"));
                    let eles = element.body().unwrap().children();
                    for ind in 0..eles.length() {
                        let node = eles.get_with_index(ind).unwrap();
                        let vnode = Html::VRef(node.into());
                        parts.push(vnode);
                    }
                } else {
                    log::debug!("{}", &format!("failed to parsed markdown into html"));
                    let node = html! { <p> {"the markdown file is not parsed"} </p> };
                    parts.push(node);
                }
            });
        }
        html! { for parts }
    }
}
