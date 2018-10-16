git branch -d gh-pages
git branch gh-pages
git checkout gh-pages

cargo build
if [ $? -ne 0 ]
then
    echo "Build failed"
    exit 1
fi

target/debug/redsystem -f -o www -b "/redsystem"
if [ $? -ne 0 ]
then
    echo "Pre-Commit Error: www/ generation failed"
    exit 1
fi
git add www/ && git commit -m "updating gh-pages"
#git push --set-upstream origin gh-pages
git subtree push --prefix www/ origin gh-pages
git checkout master

