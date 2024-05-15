declare module "*.vue" {
	import { DefineComponent } from "vue";
	// biome-ignore lint/complexity/noBannedTypes: reason
	const component: DefineComponent<{}, {}, any>;
	export default component;
}
