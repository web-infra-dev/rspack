import react from 'react';
import remote from 'remote';

global.react = react;
global.remote = remote;

import('./lazy-module').then(r=>{
	console.log('lazy module: ',r)
})

export const ok = true;
