#!/bin/bash
DIR="$(dirname $0)"
NODE_BIN=`npm bin`

rm "$DIR"/files/*.js*

if [ ! -f "$NODE_BIN/babel" ]; then
  npm install --no-save @babel/cli @babel/preset-env
fi
"$NODE_BIN/babel" "$DIR/files" --config-file "./$DIR/babel.config.js" --source-maps -d "$DIR/files"

# Strip the sourceMappingURL to prevent rollup from auto collapsing sourcemaps
for f in $DIR/files/*.js; do
  sed '$d' $f > $f.tmp
  mv $f.tmp $f
done

npx rollup -i "$DIR/files/index.js" -f cjs -o "$DIR/files/bundle.js" --sourcemap
npx prettier "$DIR/files/*.map" --parser json --write
