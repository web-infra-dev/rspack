import { EventEmitter } from "node:events";

export default class Foo extends EventEmitter {}

import("./b.js");
