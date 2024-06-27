export default function () {
	import(/* webpackPrefetch: true, webpackChunkName: "chunk-a" */ "./chunk-a");
	import(/* webpackPreload: true, webpackChunkName: "chunk-b" */ "./chunk-b");
	import(/* webpackPrefetch: 10, webpackChunkName: "chunk-c" */ "./chunk-c");
}
