export default /*#__PURE__*/ _defineComponent({
	__name: "App",
	props: {
		foo: { type: String, required: true },
	},
	setup(__props, { expose: __expose }) {
		__expose();

		const props = __props;

		const count = ref < number > 0;

		const __returned__ = { props, count };
		Object.defineProperty(__returned__, "__isScriptSetup", {
			enumerable: false,
			value: true,
		});
		return __returned__;
	},
});
