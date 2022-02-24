use crate::BlogMeta;
use crate::ParseActContext;
use crate::Route;
use yew::prelude::*;
use yew_router::components::Link;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
    pub title: String,
}

pub struct BlogCard {
    meta: BlogMeta,
}

impl Component for BlogCard {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (parser, _) = ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        log::debug!(
            "posts len: {}, indexs: {:?}",
            parser.len(),
            parser.inner().indexs
        );
        let meta = parser.get_meta(&ctx.props().id).unwrap().clone();
        Self { meta }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let (parser, _) = ctx
            .link()
            .context::<ParseActContext>(Callback::noop())
            .expect("Parser Context not found");
        self.meta = parser.get_meta(&ctx.props().id).unwrap().clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { meta } = self;
        html! {
            <div class="card">
                <div class="card-image">
                    <figure class="image is-2by1">
                        <img alt="post's image" src={meta.hero.clone()} loading="lazy" />
                    </figure>
                </div>
                <div class="card-content">
                    <Link<Route> classes={classes!("title", "is-5" )} to={Route::Post { id: meta.id, title: meta.title.clone() }}>
                        { &meta.title.replace("-", " ") }
                    </Link<Route>>
                </div>
            </div>
        }
    }
}
