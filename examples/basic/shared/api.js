// Shared API utilities
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

export const createApiClient = (baseUrl, headers) => {
  return new ApiClient(baseUrl, headers);
};

export default {
  fetchWithTimeout,
  ApiClient,
  createApiClient
};