export default function () {
	import(/* webpackPrefetch: true, webpackChunkName: "chunk1-a" */ "./a");
	import(/* webpackPrefetch: 10, webpackChunkName: "chunk1-b" */ "./b");
}
