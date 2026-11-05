export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const token = localStorage.getItem('farmlink_jwt')
  const headers = new Headers(options.headers || {})

  if (token && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  const url = endpoint.startsWith('http')
    ? endpoint
    : `http://localhost:8088${endpoint}`

  const controller = new AbortController()
  // Uploads get 5 min; regular requests timeout after 1.5s to allow fast offline fallback
  const timeoutMs = options.body instanceof FormData ? 300_000 : 1500
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs)

  try {
    const res = await fetch(url, {
      ...options,
      headers,
      signal: options.signal || controller.signal,
    })

    if (!res.ok) {
      const errorBody = await res.json().catch(() => ({ error: res.statusText }))
      const err: any = new Error(errorBody.error || `HTTP error ${res.status}`)
      err.status = res.status
      throw err
    }

    if (res.status === 204) {
      return {} as T
    }

    return res.json()
  } finally {
    clearTimeout(timeoutId)
  }
}
