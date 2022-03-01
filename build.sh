[ -z posts/.nojekyll ] && echo "post directory must constains .nojekyll file to disable jekyll SEO" && touch posts/.nojekyll
path=$(cat src/constant.rs | grep SUBPATH | grep ^pub | sed "s/^.*=//g" | sed "s/\"//g" | sed "s/\///g" | sed "s/;//g")
[ ! -z $path ] && echo "Compiled With Sub-Path: $path"
target=$( echo $path | sed "s/\///g") 
[ ! -z $path ] && sed -i "s@<base data-trunk-public-url \/.*>@<base data-trunk-public-url \/$target\/>@g" index.html || sed -i "s@<base data-trunk-public-url \/.*>@<base data-trunk-public-url \/>@g" index.html 
[ ! -z $path ] && rm -rf $target dist/ 
[ ! -z $path ] && trunk build --public-url $path --release || trunk build --release && touch dist/.nojekyll
[ ! -z $path ] && mv dist/ $target && touch $target/.nojekyll 
