"use server";

import remoteButton from "remote/Button";
import { sharedAction, sharedValue } from "shared-rsc";

export async function consumeRemoteAndShared() {
	await sharedAction();
	return `${sharedValue}:${typeof remoteButton}`;
}
