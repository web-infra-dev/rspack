module.exports = {
	findBundle(i) {
		switch (i) {
			case 0:
				return `bundle1.js`;
			case 1:
				return `bundle2.js`;
		}
	}
};
