import { describe, it, expect } from 'vitest'
import { formatBytes, formatDate } from './utils'

describe('utils', () => {
  it('formats byte edge cases correctly', () => {
    expect(formatBytes(0)).toBe('0 B')
    expect(formatBytes(-100)).toBe('0 B')
    expect(formatBytes(1024)).toBe('1 KB')
    expect(formatBytes(1048576)).toBe('1 MB')
    expect(formatBytes(52428800)).toBe('50 MB')
  })

  it('formats dates consistently', () => {
    const formatted = formatDate('2026-11-05T12:00:00Z')
    expect(formatted).toBeDefined()
    expect(formatted.length).toBeGreaterThan(0)
  })
})
