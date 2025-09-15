// @ts-nocheck
module.exports = function supportsOptionalCatchBinding() {
	try {
		eval("try {} catch {}");
		return true;
	} catch (e) {
		return false;
	}
};
