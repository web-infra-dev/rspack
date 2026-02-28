#!/usr/bin/env node

import os from "os";
import a from "./a.js";

export function hello() {
	console.log(a);
	return "hello from rslib hashbang" + os.platform();
}
