import { useEffect, useState } from 'react'
import { api } from './api'
import { mediaKind } from './fileIcons'
import type { CoverInfo } from '@/types/api'

const cache = new Map<string, CoverInfo | null>()
const listeners = new Map<string, Set<() => void>>()
const queue = new Map<string, { title: string; kind: string }>()
let timer: ReturnType<typeof setTimeout> | null = null

function keyFor(title: string): string {
  return title.trim().toLowerCase()
}

function notify(key: string) {
  for (const listener of listeners.get(key) ?? []) listener()
}

function schedule() {
  if (timer) return
  timer = setTimeout(() => {
    timer = null
    void flush()
  }, 80)
}

async function flush() {
  const entries = [...queue.entries()].slice(0, 40)
  queue.clear()
  if (entries.length === 0) return

  try {
    const covers = await api.covers(entries.map(([, item]) => item))
    for (const cover of covers) {
      const key = keyFor(cover.query)
      cache.set(key, cover)
      notify(key)
    }
    for (const [key] of entries) {
      if (!cache.has(key)) {
        cache.set(key, null)
        notify(key)
      }
    }
  } catch {
    for (const [key] of entries) {
      cache.set(key, null)
      notify(key)
    }
  }
  schedule()
}

/** Batched, deduplicated cover lookup. Covers are optional decoration. */
export function useCover(
  title: string | null | undefined,
  kind?: string,
  enabled = true,
): CoverInfo | null {
  const clean = title?.trim() ?? ''
  const resolvedKind = kind ?? (clean ? mediaKind(clean) : '')
  const key = keyFor(clean)
  const [cover, setCover] = useState<CoverInfo | null>(() => cache.get(key) ?? null)

  useEffect(() => {
    if (!clean || !enabled) {
      setCover(cache.get(key) ?? null)
      return
    }
    if (cache.has(key)) {
      setCover(cache.get(key) ?? null)
      return
    }

    const listener = () => setCover(cache.get(key) ?? null)
    const set = listeners.get(key) ?? new Set<() => void>()
    set.add(listener)
    listeners.set(key, set)
    queue.set(key, { title: clean, kind: resolvedKind })
    schedule()

    return () => {
      set.delete(listener)
      if (set.size === 0) listeners.delete(key)
    }
  }, [clean, enabled, key, resolvedKind])

  return cover
}
