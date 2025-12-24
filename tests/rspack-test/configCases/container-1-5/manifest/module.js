import react from 'react';
import { loadRemote } from 'mf';
import remote from '@remote/alias';
import List from '@remote/alias/List';
import Button from '@scope-scope/ui/Button';

global.react = react;
global.remote = remote;
global.dynamicRemote = loadRemote('@dynamic-remote/alias');
global.Button = Button;
global.List = List

import('./lazy-module').then(r=>{
	console.log('lazy module: ',r)
})

export const ok = true;
