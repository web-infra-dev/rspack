export default function () {
	import(/* webpackPreload: true, webpackChunkName: "chunk1-a" */ "./a");
	import(/* webpackPreload: 10, webpackChunkName: "chunk1-b" */ "./b");
}
