import { label } from './vendor.js';

globalThis.loadAsync = () => import(/* webpackChunkName: "async" */ './async.js');
console.log(label);
