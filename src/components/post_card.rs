use crate::BlogMeta;
use crate::ParseActContext;
use crate::Route;
use yew::prelude::*;
use yew_router::components::Link;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
    pub title: String,
    pub display: String,
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { meta } = self;
        let display = &ctx.props().display;
        //let display = use_context::<post_list::Display>().expect("Display Not Found");
        if display == "gridCard" {
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
        } else if display == "listTile" {
            html! {
                <>
                    <div class="level" style="margin-bottom:0">
                        <div class="level-left">
                                <div class="level-item">
                                        <figure class="image is-64x64" >
                                                <img style="width:100%;height:100%"  src={meta.hero.clone()} alt="post's image" />
                                        </figure>
                                </div>
                                <div class="level-item">
                                    <Link<Route> classes={classes!("title", "is-5" )} to={Route::Post { id: meta.id, title: meta.title.clone() }}>
                                        { &meta.title.replace("-", " ") }
                                    </Link<Route>>
                                </div>
                        </div>
                    </div>
                    <hr class="dotted" style="margin:0.5rem 0"/>

                </>
            }
        } else {
            html! {}
        }
    }
}
