#!/bin/bash
git clone ssh://git@github.com/open-rmf/traffic_editor_iii temp-deploy-checkout
cd temp-deploy-checkout
git checkout --orphan gh-pages
git reset
scripts/build-web.sh
git add -f web
cp web/root_index.html ./index.html
git add index.html
git commit -a -m "publish to github pages"
git push origin gh-pages --force
