const { registerClientReference } = require("react-server-dom-rspack/server");
module.exports = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "default");
