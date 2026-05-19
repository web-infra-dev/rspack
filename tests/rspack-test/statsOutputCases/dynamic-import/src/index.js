const page = 'home';

import(/* webpackChunkName: 'pages/[request]' */ `./pages/${page}`)
	.then(({ default: init }) => init());
