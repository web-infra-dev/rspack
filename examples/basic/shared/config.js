// Shared configuration module
export const API_ENDPOINTS = {
	users: "/api/users",
	posts: "/api/posts",
	auth: "/api/auth"
};

export const DEFAULT_TIMEOUT = 5000;
export const MAX_RETRIES = 3;

export const getApiUrl = endpoint => {
	return `${process.env.API_BASE_URL || "http://localhost:3000"}${endpoint}`;
};

export default {
	API_ENDPOINTS,
	DEFAULT_TIMEOUT,
	MAX_RETRIES,
	getApiUrl
};
