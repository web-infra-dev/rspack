let a = {
	data: [1, 2, 3],
	[Symbol.iterator]() {
		let index = 0;
		let data = this.data;
		return {
			next() {
				if (index >= data.length) {
					return { done: true }
				}
				return {
					done: false,
					value: data[index++]
				}
			}
		}
	}
}

