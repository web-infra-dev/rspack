'use strict';

const cp = require('child_process');
const examples = require('./examples');
const path = require('path');
const list = examples
  .concat(examples.filter((dirname) => dirname.includes('persistent-caching')))
  .filter((x) => {
    const basename = path.basename(x);
    return !basename.startsWith('.');
  })
  .forEach(function (dirname) {
    console.log(dirname);
    const build_path = `${dirname}/build.js`;
      cp.execSync(`node ${build_path}`, {
        stdio: 'inherit'
      }); // use child-process to avoid require cache
   
  });
