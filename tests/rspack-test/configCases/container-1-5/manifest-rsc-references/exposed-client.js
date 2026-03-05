"use client";

import remoteButton from "remote/Button";
import { SharedClientComponent, sharedAction, sharedValue } from "shared-rsc";

export default function ExposedButton() {
	return `${sharedValue}:${typeof remoteButton}:${typeof SharedClientComponent}`;
}

export async function exposedAction() {
	"use server";
	await sharedAction();
	return sharedValue;
}
