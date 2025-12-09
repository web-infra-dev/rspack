import { loadRemote } from 'mf';
import xreact from 'xreact';
import dep1 from '@scope-sc/dep1';
import dep2 from '@scope-sc2/dep2';
import remote from '@remote/alias';

global.xreact = xreact;
global.remote = remote;
global.dep1 = dep1;
global.dep2 = dep2;

loadRemote('dynamic-remote');

import('@remote/alias/Button').then(r=>{
	console.log('@remote/alias/Button: ',r)
})

import('./lazy-module').then(r=>{
	console.log('lazy module: ',r)
})

export const ok = true;
