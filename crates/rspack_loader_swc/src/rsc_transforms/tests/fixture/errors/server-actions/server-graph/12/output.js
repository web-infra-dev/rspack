import { registerClientReference } from "react-server-dom-rspack/server";
import * as React from "react";
const resources = (__rspack_rsc_manifest__.clientManifest?.["/app/item.js"]?.cssFiles ?? []).map(function(href) {
    return React.createElement("link", {
        ...__rspack_rsc_manifest__.cssLinkProps,
        key: href,
        rel: "stylesheet",
        href: href
    });
});
const Ref1 = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/app/item.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/app/item.js", "foo");
export const foo = resources.length ? (props)=>React.createElement(React.Fragment, null, resources, React.createElement(Ref1, props)) : Ref1;
