import { util } from './util';
import { a } from './a';
import { b } from './b';

export const load = () => import(/* webpackChunkName: "leaf" */ './leaf');
export const value = util(a + b);
