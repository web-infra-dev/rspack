import {
	prop as importedProp, 
	staticProp as importedStaticProp
} from "./module";

export const staticProp = "value";
export const prop = "value";

export class A {
	static staticProp = staticProp;
	static [staticProp] = staticProp;
	prop = prop;
	[prop] = prop;
}

export class B {
	static staticProp = importedStaticProp;
	static [importedStaticProp] = importedStaticProp;
	prop = importedProp;
	[importedProp] = importedProp;
}
