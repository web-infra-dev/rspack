import react from 'react';

global.react = react;

import('./lazy-module').then(r=>{
	console.log('lazy module: ',r)
})

export const ok = true;
