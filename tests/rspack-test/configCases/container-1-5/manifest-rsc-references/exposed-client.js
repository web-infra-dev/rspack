"use client";

import remoteButton from "remote/Button";
import { sharedAction, sharedValue } from "shared-rsc";

export default function ExposedButton() {
	return `${sharedValue}:${typeof remoteButton}`;
}

export async function exposedAction() {
	"use server";
	await sharedAction();
	return sharedValue;
}
