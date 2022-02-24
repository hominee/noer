use crate::components::pagination::PageQuery;
use crate::components::pagination::Pagination;
use crate::components::post_card::BlogCard;
use crate::constant::ITEMS_PER_PAGE;
use crate::parser::ParseAct;
use crate::ParseActContext;
use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

pub enum Msg {
    PageUpdated,
    LoadPageUpdate,
}

pub struct PostList {
    page: u64,
    _listener: LocationHandle,
}

fn current_page(ctx: &Context<PostList>) -> u64 {
    let location = ctx.link().location().unwrap();

    location.query::<PageQuery>().map(|it| it.page).unwrap_or(1)
}

impl Component for PostList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let listener = ctx
            .link()
            .add_location_listener(link.callback(move |_| Msg::PageUpdated))
            .unwrap();

        Self {
            page: current_page(ctx),
            _listener: listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PageUpdated => self.page = current_page(ctx),
            Msg::LoadPageUpdate => self.page += 1,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = self.page;

        let (parser, _) = ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        let link = ctx.link().clone();
        let all_pages = parser.all_page();
        let total_pages = parser.total_page();
        let onclick = Callback::from(move |_| {
            parser.dispatch(ParseAct::MoreBlogMeta);
            link.send_message(Msg::LoadPageUpdate);
        });

        html! {
            <div class="section container">
                { self.view_posts(ctx) }
                <Pagination
                    {page}
                    total_pages={total_pages}
                    route_to_page={Route::Home}
                />
            {
                if total_pages == page && all_pages > page {
                    html!{
                        <>
                        <button class= "button is-medium is-fullwidth is-inverted is-responsive" {onclick}> { "Load More Page" }  </button>
                        </>
                    }
                } else {
                    html!{}
                }
            }
            </div>
        }
    }
}
impl PostList {
    fn view_posts(&self, _ctx: &Context<Self>) -> Html {
        let (parse, _) = _ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        let start_seed = (self.page - 1) * ITEMS_PER_PAGE;
        let mut cards = Vec::new();
        for ind in 0..ITEMS_PER_PAGE {
            let index = ind + start_seed;
            if index as usize > parse.ids.len() - 1 || parse.ids.is_empty() {
                break;
            }
            let id = parse.ids[index as usize];
            let meta = parse.get_meta(&id).unwrap();
            let item = html! {
                <li class="list-item mb-5">
                    <BlogCard id={id} title={meta.title.clone()} />
                </li>
            };
            cards.push(item);
        }
        let mut cards = cards.into_iter();
        html! {
            <div class="columns">
                <div class="column">
                    <ul class="list">
                        { for cards.by_ref().take(ITEMS_PER_PAGE as usize / 3) }
                    </ul>
                </div>
                <div class="column">
                    <ul class="list">
                        { for cards.by_ref().take(ITEMS_PER_PAGE as usize / 3) }
                    </ul>
                </div>
                <div class="column">
                    <ul class="list">
                        { for cards }
                    </ul>
                </div>
            </div>
        }
    }
}
