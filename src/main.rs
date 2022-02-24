mod components;
mod constant;
mod content;
mod pages;
mod parser;

use crate::content::Blog;
use crate::content::BlogMeta;
use crate::pages::post::FetchState;
use crate::parser::read_dir;
use parser::Parser;
use yew::prelude::*;
use yew_router::prelude::*;

use pages::{page_not_found::PageNotFound, post::Post, post_list::PostList};
use yew::html::Scope;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/:id/:title")]
    Post { id: u64, title: String },
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub type ParseActContext = UseReducerHandle<Parser>;
use crate::parser::ParseAct;

#[function_component(App)]
pub fn app() -> Html {
    let ctx = use_reducer(|| Parser::new());

    html! {
        <ContextProvider<ParseActContext> context={ctx}>
            <Model />
        </ContextProvider<ParseActContext>>
    }
}

pub enum Msg {
    LoadBlogMeta,
    Ready,
    Notified,
}

pub struct Model {
    state: FetchState<()>,
    notified: bool,
}
impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        use std::sync::Once;
        static mut INIT: Once = Once::new();
        unsafe {
            INIT.call_once(|| {
                _ctx.link().send_message(Msg::LoadBlogMeta);
            });
        }
        Self {
            state: FetchState::NotFetching,
            notified: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (parser, _) = _ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        match msg {
            Msg::Notified => {
                self.notified = true;
                true
            }
            Msg::Ready => {
                self.state = FetchState::Success(());
                true
            }
            Msg::LoadBlogMeta => {
                _ctx.link().send_future(async move {
                    let paths = read_dir().await;
                    log::trace!("markdown index: {:?}", paths);
                    parser.dispatch(ParseAct::BlogPath(paths));
                    parser.dispatch(ParseAct::MoreBlogMeta);
                    Msg::Ready
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::Notified);
        match self.state {
            FetchState::NotFetching => html! {
                <>
                        { self.view_nav(ctx.link()) }
                { if !self.notified  {
                     html!{
                        <div class="container mt-24 notification is-light">
                            <button class="delete" {onclick}> </button>
                            <br />
                            <strong class="content is-large">{ " Blogs Not Fetched" }</strong>
                            <br />
                        </div>
                     }
                } else {
                    html!{}
                }
                }
                </>
            },
            FetchState::Fetching => html! {
                <>
                        { self.view_nav(ctx.link()) }
                { if !self.notified {
                    html!{
                        <div class="container mt-24 notification is-info">
                            <button class="delete" {onclick}> </button>
                            <br />
                            <strong class="content is-large">{ "Loading Blog " }</strong>
                            <br />
                        </div>
                    }
                   } else {
                       html! {}
                 }
                }
                </>
            },
            FetchState::Failed(_) => html! {
                <>
                { self.view_nav(ctx.link()) }
                { if !self.notified {
                   html!{
                    <div class="container mt-24 notification is-danger">
                        <button class="delete" {onclick}> </button>
                        <br />
                        <strong class="is-large">{ " Blog Loaded Failed" }</strong>
                    </div>
               } } else { html!{} }
                }
                </>
            },
            FetchState::Success(_) => {
                html! {
                    <BrowserRouter>
                        { self.view_nav(ctx.link()) }

                    <main>
                        <Switch<Route> render={Switch::render(switch)} />
                    </main>
                        <footer class="footer">
                            <div class="content has-text-centered">
                              { constant::SITE_DESCRIPTION }
                            </div>
                        </footer>
                        </BrowserRouter>
                }
            }
        }
    }
}
impl Model {
    fn view_user_info(&self) -> Html {
        let s = constant::USER_INFO;
        let mut infos = Vec::new();
        s.split("\n")
            .map(|e| e.trim())
            .filter(|e| e.len() != 0)
            .for_each(|e| {
                let pairs: Vec<&str> = e
                    .splitn(2, ":")
                    .map(|e| e.trim().trim_end_matches(","))
                    .collect();
                if pairs.len() == 2 {
                    log::debug!("key: {}, value: {}", pairs[0], pairs[1]);
                    let item = if pairs[0] == "email" {
                        html! {
                            <a class="navbar-item" href={format!("mailto:{}", pairs[1])} >
                                { pairs[0] }
                            </a>
                        }
                    } else {
                        html! {
                            <a class="navbar-item" target="_blank" href={pairs[1]} >
                                { pairs[0] }
                            </a>
                        }
                    };
                    infos.push(item);
                }
            });
        html! {
            <div class="navbar-dropdown">
              { for infos }
            </div>
        }
    }

    fn view_nav(&self, _link: &Scope<Self>) -> Html {
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                    <h1 class="navbar-item is-size-3">{ constant::SITE_NAME }</h1>
                    </a>
                    <a class="navbar-item" href="/">
                        <figure class="image is-rounded pr-3">
                            <img src={constant::LOGO_PIC} class="image"/>
                        </figure>
                    </a>
                </div>
                <div class="navbar-end">
                    <div class="navbar-item" >
                    <div class="field is-grouped">
                        <a  href="/">
                            <figure class="image is-rounded pr-3">
                                <img style="width:auto;" src={ constant::AVATR_PIC} />
                            </figure>
                        </a>
                        <div class="navbar-item has-dropdown is-hoverable">
                        <a class="title is-5" href="/">
                            { constant::ADMIN }
                        </a>
                          { self.view_user_info() }
                        </div>
                    </div>
                    </div>
                </div>
            </nav>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes.clone() {
        Route::Post { id, title } => {
            html! { <Post id={id} title={title} /> }
        }
        Route::Home => {
            html! { <PostList /> }
        }
        Route::NotFound => {
            html! { <PageNotFound /> }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    log::debug!("before parse");
    yew::start_app::<App>();
}
