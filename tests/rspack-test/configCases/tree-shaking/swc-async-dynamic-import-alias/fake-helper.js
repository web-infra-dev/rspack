export function _(generator) {
	return async function (...args) {
		const iterator = generator.apply(this, args);
		const yielded = iterator.next();
		const namespace = await yielded.value;
		return {
			namespace,
			value: iterator.next({ default: 42 }).value
		};
	};
}
