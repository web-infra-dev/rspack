"use server";

import { sharedAction, sharedValue } from "shared-rsc";

export async function consumeShared() {
	await sharedAction();
	return sharedValue;
}
