const toArray = a => Array.from(a);
const toSet = a => new Set(a);
const unify = a => toArray(toSet(a));
const toMatcher = a => {
	if (a instanceof RegExp) {
		return i => a.test(i);
	}
	if (typeof a === "string") {
		return i => i.startsWith(a);
	}
	throw new Error("Expected `RegExp` or `string`, got " + typeof a);
};
const zip = (a, b) => a.map((a, index) => [a, b[index]]);

const intersection = (a, b) => a.filter(item => b.includes(item));
const union = (a, b) => toArray(toSet(a.concat(b)));

const retain = fn => arr => toArray(arr).filter(fn);

const each = fn => item => fn(item);
const not = fn => item => !fn(item);

const includes = (arr, item) => arr.includes(item);
const includedIn = arr => item => includes(arr, item);

const matchedWith = rule => item => toMatcher(rule)(item);
const matchedInAll = arr => item =>
	arr.map(toMatcher).every(matcher => matcher(item));
const matchedInAny = arr => item =>
	arr.map(toMatcher).some(matcher => matcher(item));

module.exports = {
	toArray,
	toSet,
	unify,
	zip,

	intersection,
	union,

	retain,

	each,
	not,

	includedIn,

	matchedWith,
	matchedInAll,
	matchedInAny
};
