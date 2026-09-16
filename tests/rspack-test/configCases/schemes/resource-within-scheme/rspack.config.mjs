/** @type {import('webpack').Configuration} */
export default {
  entry: ['data:text/javascript,import "polyfill";', './index.js'],
};
