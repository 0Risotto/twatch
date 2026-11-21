import { useCallback, useEffect, useRef } from 'react'

const BUCKET_SECONDS = 5
const CAPTURE_WIDTH = 176
const SEEK_TIMEOUT = 4_000
const LOAD_TIMEOUT = 6_000
const CACHE_LIMIT = 240
const PREFETCH_LIMIT = 6

interface CaptureJob {
  key: string
  time: number
  resolve: (url: string | null) => void
}

function bucketOf(time: number): number {
  return Math.max(0, Math.floor(time / BUCKET_SECONDS) * BUCKET_SECONDS)
}

function keyOf(streamUrl: string, bucket: number): string {
  return `${streamUrl}#${bucket}`
}

function createWorker(): HTMLVideoElement {
  const video = document.createElement('video')
  video.preload = 'metadata'
  video.muted = true
  video.playsInline = true
  video.setAttribute('aria-hidden', 'true')
  video.style.cssText =
    'position:fixed;left:-9999px;top:0;width:2px;height:2px;opacity:0;pointer-events:none'
  document.body.appendChild(video)
  return video
}

function destroyWorker(video: HTMLVideoElement | null): void {
  if (!video) return
  video.pause()
  video.removeAttribute('src')
  video.load()
  video.remove()
}

function waitForReady(
  video: HTMLVideoElement,
  minReadyState: number,
  timeout: number,
): Promise<boolean> {
  if (video.readyState >= minReadyState) return Promise.resolve(true)
  return new Promise(resolve => {
    let settled = false
    const finish = (ok: boolean) => {
      if (settled) return
      settled = true
      window.clearTimeout(timer)
      video.removeEventListener('loadedmetadata', onLoaded)
      video.removeEventListener('loadeddata', onLoaded)
      video.removeEventListener('canplay', onLoaded)
      video.removeEventListener('error', onError)
      resolve(ok)
    }
    const onLoaded = () => {
      if (video.readyState >= minReadyState) finish(true)
    }
    const onError = () => finish(false)
    const timer = window.setTimeout(() => finish(false), timeout)
    video.addEventListener('loadedmetadata', onLoaded)
    video.addEventListener('loadeddata', onLoaded)
    video.addEventListener('canplay', onLoaded)
    video.addEventListener('error', onError)
  })
}

function seekVideo(video: HTMLVideoElement, time: number, timeout: number): Promise<boolean> {
  const duration = Number.isFinite(video.duration) ? video.duration : 0
  if (duration <= 0) return Promise.resolve(false)
  const target = Math.max(0, Math.min(time, Math.max(0, duration - 0.1)))
  if (video.readyState >= 2 && Math.abs(video.currentTime - target) < 0.3) {
    return Promise.resolve(true)
  }

  return new Promise(resolve => {
    let settled = false
    const finish = (ok: boolean) => {
      if (settled) return
      settled = true
      window.clearTimeout(timer)
      video.removeEventListener('seeked', onSeeked)
      video.removeEventListener('timeupdate', onTime)
      video.removeEventListener('error', onError)
      resolve(ok)
    }
    const onSeeked = () => finish(true)
    const onTime = () => {
      if (video.readyState >= 2 && Math.abs(video.currentTime - target) < 0.5) finish(true)
    }
    const onError = () => finish(false)
    const timer = window.setTimeout(() => finish(false), timeout)

    // Attach first: a fast range request can fire `seeked` before we set up.
    video.addEventListener('seeked', onSeeked)
    video.addEventListener('timeupdate', onTime)
    video.addEventListener('error', onError)
    try {
      video.currentTime = target
    } catch {
      finish(false)
    }
  })
}

/**
 * On-demand seek-preview thumbnails with latest-wins scheduling.
 *
 * The previous position is superseded as soon as the pointer moves, so a
 * drag always previews the latest bucket instead of waiting behind a queue.
 * Frames are cached per five-second bucket and evicted in insertion order.
 */
export function useThumbnails(streamUrl: string | null) {
  const streamRef = useRef(streamUrl)
  streamRef.current = streamUrl

  const worker = useRef<HTMLVideoElement | null>(null)
  const cache = useRef(new Map<string, string>())
  const order = useRef<string[]>([])
  const inflight = useRef(new Map<string, Promise<string | null>>())
  const userJob = useRef<CaptureJob | null>(null)
  const prefetchJobs = useRef<CaptureJob[]>([])
  const running = useRef(false)
  const canvas = useRef<HTMLCanvasElement | null>(null)
  const generation = useRef(0)

  useEffect(() => {
    const current = generation.current + 1
    generation.current = current
    cache.current.clear()
    order.current = []
    inflight.current.clear()
    for (const job of prefetchJobs.current.splice(0)) job.resolve(null)
    if (userJob.current) {
      userJob.current.resolve(null)
      userJob.current = null
    }
    return () => {
      if (generation.current === current) {
        destroyWorker(worker.current)
        worker.current = null
      }
    }
  }, [streamUrl])

  const remember = useCallback((key: string, url: string) => {
    cache.current.set(key, url)
    order.current.push(key)
    while (order.current.length > CACHE_LIMIT) {
      const oldest = order.current.shift()
      if (oldest) cache.current.delete(oldest)
    }
  }, [])

  const capture = useCallback(async (time: number): Promise<string | null> => {
    const url = streamRef.current
    if (!url) return null
    const generationAtStart = generation.current
    if (!worker.current) worker.current = createWorker()
    const video = worker.current
    if (video.getAttribute('src') !== url) {
      video.src = url
      video.load()
    }
    if (!(await waitForReady(video, 1, LOAD_TIMEOUT))) {
      video.load()
      return null
    }
    if (generationAtStart !== generation.current) return null
    if (!(await seekVideo(video, time, SEEK_TIMEOUT))) return null
    if (!(await waitForReady(video, 2, LOAD_TIMEOUT))) return null
    if (generationAtStart !== generation.current || !video.videoWidth) return null

    if (!canvas.current) canvas.current = document.createElement('canvas')
    const element = canvas.current
    const height = Math.max(1, Math.round((CAPTURE_WIDTH * video.videoHeight) / video.videoWidth))
    element.width = CAPTURE_WIDTH
    element.height = height
    const context = element.getContext('2d')
    if (!context) return null
    context.drawImage(video, 0, 0, CAPTURE_WIDTH, height)
    try {
      return element.toDataURL('image/jpeg', 0.72)
    } catch {
      return null
    }
  }, [])

  const run = useCallback(async () => {
    if (running.current) return
    running.current = true
    try {
      while (userJob.current || prefetchJobs.current.length > 0) {
        const job = userJob.current ?? prefetchJobs.current.shift()
        if (!job) break
        if (job === userJob.current) userJob.current = null
        let url: string | null = null
        try {
          url = await capture(job.time)
          if (url) remember(job.key, url)
        } catch {
          url = null
        }
        job.resolve(url)
      }
    } finally {
      running.current = false
    }
  }, [capture, remember])

  const enqueue = useCallback(
    (time: number, priority: boolean): Promise<string | null> => {
      const url = streamRef.current
      if (!url || !Number.isFinite(time) || time < 0) return Promise.resolve(null)
      const bucket = bucketOf(time)
      const key = keyOf(url, bucket)
      const cached = cache.current.get(key)
      if (cached) return Promise.resolve(cached)
      const pending = inflight.current.get(key)
      if (pending) return pending

      const promise = new Promise<string | null>(resolve => {
        const job = { key, time: bucket, resolve }
        if (priority) {
          // The newest pointer position wins; discard older waiting work.
          if (userJob.current) userJob.current.resolve(null)
          userJob.current = job
        } else {
          if (prefetchJobs.current.length >= PREFETCH_LIMIT) {
            prefetchJobs.current.shift()?.resolve(null)
          }
          prefetchJobs.current.push(job)
        }
        void run()
      })
      inflight.current.set(key, promise)
      void promise.finally(() => {
        if (inflight.current.get(key) === promise) inflight.current.delete(key)
      })
      return promise
    },
    [run],
  )

  const getThumbnail = useCallback((time: number) => enqueue(time, true), [enqueue])
  const prefetchThumbnail = useCallback(
    (time: number) => {
      void enqueue(time, false)
    },
    [enqueue],
  )
  const getCachedThumbnail = useCallback((time: number): string | null => {
    const url = streamRef.current
    if (!url || !Number.isFinite(time) || time < 0) return null
    return cache.current.get(keyOf(url, bucketOf(time))) ?? null
  }, [])

  return { getThumbnail, prefetchThumbnail, getCachedThumbnail }
}
