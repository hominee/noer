
[ ! -z $1 ] && echo "github page sub path: $1"
trunk build --release
sed -i "s/href=\"\//href=\"{\/$1}\//" dist/index.html && sed -i "s/'\/index-/'{\/$1}\/index-/g" dist/index.html
rm -rf docs/ 
mv dist/ docs/
