import $ from 'jquery';
import _ from 'underscore';

export default 'async-bar';

export const asyncBar = () => {
  import(/* webpackChunkName: "async-async-bar" */ './async-async-bar.js');
  import(/* webpackChunkName: "async-async-bar-two" */ './async-async-bar-two.js');
};
