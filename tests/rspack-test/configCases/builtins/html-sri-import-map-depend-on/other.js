import { loadVendor } from "./vendor.js";

loadVendor().then(console.log);
import(/* webpackChunkName: "other-lazy" */ "./other-lazy.js").then(
	console.log
);
