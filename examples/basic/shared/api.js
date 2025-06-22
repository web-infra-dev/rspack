// Shared API utilities - testing various export scenarios

// Unused export (not imported directly, used by ApiClient)
export const fetchWithTimeout = async (url, options = {}, timeout = 5000) => {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);
    throw error;
  }
};

// Unused export (not imported directly, used by createApiClient)
export class ApiClient {
  constructor(baseUrl, headers = {}) {
    this.baseUrl = baseUrl;
    this.headers = headers;
  }

  async get(endpoint) {
    return fetchWithTimeout(`${this.baseUrl}${endpoint}`, {
      headers: this.headers
    });
  }

  async post(endpoint, data) {
    return fetchWithTimeout(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...this.headers
      },
      body: JSON.stringify(data)
    });
  }
}

// Used export (imported in index.js)
export const createApiClient = (baseUrl, headers) => {
  return new ApiClient(baseUrl, headers);
};

// Additional unused exports for testing
export const buildQueryString = (params) => {
  return Object.entries(params)
    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
    .join('&');
};

export class GraphQLClient {
  constructor(endpoint, headers = {}) {
    this.endpoint = endpoint;
    this.headers = headers;
  }
  
  async query(query, variables = {}) {
    return fetchWithTimeout(this.endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...this.headers
      },
      body: JSON.stringify({ query, variables })
    });
  }
}

// Default export (not imported but defined)
export default {
  fetchWithTimeout,
  ApiClient,
  createApiClient,
  buildQueryString,
  GraphQLClient
};