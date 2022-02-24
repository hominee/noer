# What's Noer

[noer](https://github.com/homelyguy/noer) is a minimal static style blog template, which is good choice for those who prefer **Markdown**, **static site**, **Web-assembly**.


See [Demo site](https://homelyguy.github.io/demo)


# How To Use
there are some requirements to be satisfied before we use it.

## Pre-requisite
- First of all, [rust](https://rust-lang.org) language and its building tool are necessary. you can install them following the description [here](https://www.rust-lang.org/tools/install);
-	Noer is built upon Web-Assembly and [yew](https://yew.rs), the [tutorial](https://yew.rs/docs/tutorial) is recommanded before getting started;

## Github Page(Optional)
By default, the compiled output is not suitable for github pages, you got to enable it in the configuration file `src/constant.rs` 
```
// by default it is false
pub const USE_GITPAGE: bool = false;
// if using github pages, specify the sub-path(ends with `/`)
pub const SUBPATH: &str = "demo/";
```

## Site Infomation
There are some basic info about the site in `stc/constant.rs`, including
- `ITEMS_PER_PAGE` number of posts card that a page to display, note that it shall be multiple of 3.
- `ADMIN` username of the ower.
- `SITE_NAME` the name you want call the site.
- `LOGO_PIC`: the logo display in the page.
- `AVATR_PIC` avatar of the user
- `SITE_DESCRIPTION` just site Description.
- `USER_INFO` some extra infomation of the user present to viewers, such as social network, email, etc. Note that it is line-separated, each line is a key-value pair.

## Compile
```
// normal
sh ./build.sh

// github pages output
// add the sub-path
sh ./build.sh demo
```

# Known Issues
- due to the limit of yew, `late` or `katex` is not supported, see [issue](https://github.com/yewstack/yew/discussions/2446)
- some uncommon features of markdown are not supported so far (subscript/supscript, some Markup, Definition, Abbreviations)
