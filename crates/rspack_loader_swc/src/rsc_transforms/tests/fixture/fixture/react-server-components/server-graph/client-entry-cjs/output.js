const { registerClientReference } = require("react-server-dom-rspack/server");
const React = require("react");
const resources = (__rspack_rsc_manifest__.clientManifest?.["/some-project/src/some-file.js"]?.cssFiles ?? []).map(function(href) {
    return React.createElement("link", {
        ...__rspack_rsc_manifest__.cssLinkProps,
        key: href,
        rel: "stylesheet",
        href: href
    });
});
