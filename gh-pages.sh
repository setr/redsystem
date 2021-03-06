
cargo build
if [ $? -ne 0 ]
then
    echo "Build failed"
    exit 1
fi

target/debug/redsystem -f -o www -b "redsystem"
if [ $? -ne 0 ]
then
    echo "Pre-Commit Error: www/ generation failed"
    exit 1
fi
git add www/
git commit -m "Updating ghpages"
git subtree split --prefix www/ -b gh-pages
git push -f origin gh-pages:gh-pages
git branch -D gh-pages 
