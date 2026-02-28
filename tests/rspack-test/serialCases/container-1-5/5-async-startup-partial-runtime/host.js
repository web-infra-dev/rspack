import('containerA/ComponentA').then(({ default: ComponentA }) => {
	if (typeof ComponentA !== 'function') {
		throw new Error('[async-startup-partial-runtime] remote not resolved');
	}
	return ComponentA();
});
