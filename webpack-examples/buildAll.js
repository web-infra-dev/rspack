'use strict';

const cp = require('child_process');
const examples = require('./examples');
const passedList = ['asset-simple'];
examples
  .concat(examples.filter((dirname) => dirname.includes('persistent-caching')))
  .filter((x) => {
    let idx = passedList.findIndex((item) => {
      return x.includes(item);
    });
    return idx !== -1;
  })
  .forEach(function (dirname) {
    const build_path = `${dirname}/build.js`;
    console.log(build_path);
    require(build_path);
  });
