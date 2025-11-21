import { Lib as OriginLib, value } from "./lib";

export class Lib extends OriginLib {}

function foo() {
	return { OriginLib };
}

export const v = value;
