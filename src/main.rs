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
    #[at("/posts/:id/:title")]
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
}

pub struct Model {
    state: FetchState<()>,
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
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (parser, _) = _ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        match msg {
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
        let dom_parser = web_sys::DomParser::new().unwrap();
        let mut footer = Vec::new();
        let input_str = format!("<div>{}</div>", constant::SITE_DESCRIPTION);
        if let Ok(element) =
            dom_parser.parse_from_string(&input_str, web_sys::SupportedType::TextHtml)
        {
            let eles = element.body().unwrap().children();
            for ind in 0..eles.length() {
                let node = eles.get_with_index(ind).unwrap();
                let vnode = Html::VRef(node.into());
                footer.push(vnode);
            }
        }
        match self.state {
            FetchState::Success(_) => {
                html! {
                    <BrowserRouter>
                        { self.view_nav(ctx.link()) }

                    <main>
                        <Switch<Route> render={Switch::render(switch)} />
                    </main>
                        <footer class="footer">
                            <div class="content has-text-centered">
                              {
                                  html!{ for footer }
                              }
                            </div>
                        </footer>
                    </BrowserRouter>
                }
            }
            _ => html! {
                <>
                { self.view_nav(ctx.link()) }
                </>
            },
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
        let href = format!("/{}/", constant::SUBPATH.replace("/", ""));
        let logo = format!(
            "/{}/{}",
            constant::SUBPATH.replace("/", ""),
            constant::LOGO_PIC
        );
        let avatar = format!(
            "/{}/{}",
            constant::SUBPATH.replace("/", ""),
            constant::AVATR_PIC
        );
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" href={href.clone()}>
                    <h1 class="navbar-item is-size-3">{ constant::SITE_NAME }</h1>
                    </a>
                    <a class="navbar-item" href={href.clone()}>
                        <figure class="image is-rounded pr-3">
                            <img src={logo} class="image"/>
                        </figure>
                    </a>
                </div>
                <div class="navbar-end">
                    <div class="navbar-item" >
                    <div class="field is-grouped">
                        <a  href={href.clone()}>
                            <figure class="image is-rounded pr-3">
                                <img style="width:auto;" src={avatar} />
                            </figure>
                        </a>
                        <div class="navbar-item has-dropdown is-hoverable">
                        <a class="title is-5" href={href}>
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
    // set logger depends on the logger filter
    match constant::MODE {
        constant::Modes::Release => {}
        constant::Modes::Development(level) => {
            wasm_logger::init(wasm_logger::Config::new(level));
        }
    }
    yew::start_app::<App>();
}
