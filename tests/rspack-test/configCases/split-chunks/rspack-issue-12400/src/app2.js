import _ from 'underscore';

import(/* webpackChunkName: "async-bar" */ './async-bar.js').then((x) => {
  console.log.bind(x);
});
