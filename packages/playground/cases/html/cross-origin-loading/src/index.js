// Set the public path of async chunk to a cross-origin URL
__webpack_public_path__ = "https://cdn.example.com/";

// Avoid removing script tag
Node.prototype.removeChild = () => {};

import("./foo.js");
