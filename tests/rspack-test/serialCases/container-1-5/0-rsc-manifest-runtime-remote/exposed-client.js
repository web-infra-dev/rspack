"use client";

import { SharedClientComponent, sharedAction, sharedValue } from "shared-rsc";

export default function RemoteButton() {
	return `${sharedValue}:${typeof SharedClientComponent}`;
}

export async function remoteAction() {
	"use server";
	await sharedAction();
	return sharedValue;
}
