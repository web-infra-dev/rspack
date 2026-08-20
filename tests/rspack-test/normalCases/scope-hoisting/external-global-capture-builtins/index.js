import externalJSON from "json";
import externalPromise from "promise";
import externalURL from "url";
import externalURLSearchParams from "url-search-params";
import externalSymbol from "symbol";
import externalReflect from "reflect";
import externalGlobalThis from "global-this";
import {
	JSON as localJSON,
	Promise as localPromise,
	URL as localURL,
	URLSearchParams as localURLSearchParams,
	Symbol as localSymbol,
	Reflect as localReflect,
	globalThis as localGlobalThis
} from "./local";

it("should keep generated external globals out of the concatenated local scope", () => {
	expect(externalJSON.marker).toBe("global-json");
	expect(localJSON.marker).toBe("local-json");
	expect(externalPromise.marker).toBe("global-promise");
	expect(localPromise.marker).toBe("local-promise");
	expect(externalURL.marker).toBe("global-url");
	expect(localURL.marker).toBe("local-url");
	expect(externalURLSearchParams.marker).toBe("global-url-search-params");
	expect(localURLSearchParams.marker).toBe("local-url-search-params");
	expect(externalSymbol.marker).toBe("global-symbol");
	expect(localSymbol.marker).toBe("local-symbol");
	expect(externalReflect.marker).toBe("global-reflect");
	expect(localReflect.marker).toBe("local-reflect");
	expect(externalGlobalThis.marker).toBe("global-global-this");
	expect(localGlobalThis.marker).toBe("local-global-this");
});
