import { loadRemote } from 'mf';
import xreact from 'xreact';
import dep1 from '@scope-sc/dep1';
import dep2 from '@scope-sc2/dep2';
import remote from '@remote/alias';

export function App() {
	return `${dep1} ${dep2} ${remote} ${xreact}`
}

export async function AppAsync() {
	const dynamicRemote = await loadRemote('dynamic-remote');
	const Button = (await import('@remote/alias/Button')).default;
	const lazyModule = (await import('./lazy-module')).lazy;
	return `${dynamicRemote} ${Button} ${lazyModule}`
}
