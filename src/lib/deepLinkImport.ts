const IMPORT_SCHEME = "waken-wa-reporter:";
const IMPORT_HOST = "import";
const PAYLOAD_QUERY_KEYS = ["config", "payload", "data", "import"];

const LEGACY_QUERY_KEYS = {
  endpoint: ["reportEndpoint", "endpoint", "baseUrl", "url"],
  apiKey: ["apiKey", "apiToken", "token"],
  tokenName: ["tokenName", "name"],
  deviceName: ["deviceName", "device_name", "device"],
} as const;

export function isImportDeepLink(value: string): boolean {
  const url = parseUrl(value);
  return Boolean(url && url.protocol === IMPORT_SCHEME && url.hostname === IMPORT_HOST);
}

export function extractImportPayloadFromDeepLink(value: string): string | null {
  const url = parseUrl(value);
  if (!url || !isImportDeepLink(value)) {
    return null;
  }

  const queryPayload = firstQueryValue(url.searchParams, PAYLOAD_QUERY_KEYS);
  if (queryPayload) {
    return normalizePossiblyUnescapedBase64(queryPayload);
  }

  const pathPayload = url.pathname
    .split("/")
    .filter(Boolean)
    .join("/");
  if (pathPayload) {
    return decodeURIComponent(pathPayload);
  }

  const hashPayload = url.hash.startsWith("#") ? url.hash.slice(1).trim() : "";
  if (hashPayload) {
    return decodeURIComponent(hashPayload);
  }

  return legacyPayloadFromQuery(url.searchParams);
}

function parseUrl(value: string): URL | null {
  try {
    return new URL(value);
  } catch {
    return null;
  }
}

function firstQueryValue(params: URLSearchParams, keys: readonly string[]) {
  for (const key of keys) {
    const value = params.get(key)?.trim();
    if (value) {
      return value;
    }
  }
  return "";
}

function normalizePossiblyUnescapedBase64(value: string) {
  return value.replace(/ /g, "+").trim();
}

function legacyPayloadFromQuery(params: URLSearchParams): string | null {
  const endpoint = firstQueryValue(params, LEGACY_QUERY_KEYS.endpoint);
  const apiKey = firstQueryValue(params, LEGACY_QUERY_KEYS.apiKey);
  if (!endpoint && !apiKey) {
    return null;
  }

  return JSON.stringify({
    endpoint,
    apiKey,
    tokenName: firstQueryValue(params, LEGACY_QUERY_KEYS.tokenName),
    deviceName: firstQueryValue(params, LEGACY_QUERY_KEYS.deviceName),
  });
}
