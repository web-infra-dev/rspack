export * from "webpack-dev-server/client/clients/SockJSClient";

// TODO: hack providerPlugin
// @ts-ignored
__webpack_require__.$WsClient$ = require("webpack-dev-server/client/clients/SockJSClient");
