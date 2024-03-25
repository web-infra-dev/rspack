// @ts-nocheck
const urlToRelativePath = require("./urlToRelativePath");
module.exports = function createHotDocument(outputDirectory, getRequire) {
	return {
		createElement(type) {
			return {
				_type: type,
				_attrs: {},
				setAttribute(name, value) {
					this._attrs[name] = value;
				},
				// CHANGE: added support for `getAttribute` method
				getAttribute(name) {
					return this._attrs[name];
				},
				// CHANGE: added support for `removeAttribute` method
				removeAttribute(name) {
					delete this._attrs[name];
				},
				parentNode: {
					removeChild(node) {
						// ok
					}
				},
				// CHANGE: added support for css link
				sheet: {
					disabled: false
				}
			};
		},
		head: {
			// CHANGE: added support for `children` property
			children: [],
			// CHANGE: added support for `insertBefore` method
			insertBefore(element, before) {
				element.parentNode = this;
				this.children.unshift(element);
				Promise.resolve().then(() => {
					if (element.onload) {
						element.onload({ type: "load", target: element });
					}
				});
			},
			// CHANGE: enhanced `insertBefore` method
			appendChild(element) {
				element.parentNode = this;
				this.children.push(element);
				if (element._type === "script") {
					Promise.resolve().then(() => {
						getRequire()(outputDirectory, urlToRelativePath(element.src));
						if (element.onload) {
							element.onload({
								type: "load",
								target: element
							});
						}
					});
				} else {
					if (element.onload) {
						element.onload({ type: "load", target: element });
					}
				}
			},
			// CHANGE: added support for `removeChild` method
			removeChild(node) {
				const index = this.children.indexOf(node);
				this.children.splice(index, 1);
			}
		},
		getElementsByTagName(name) {
			if (name === "head") return [this.head];
			// CHANGE: added support for link tag
			if (name === "script" || name === "link")
				return this.head.children.filter(i => i._type === name);
			throw new Error("Not supported");
		}
	};
};
