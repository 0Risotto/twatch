import '@testing-library/jest-dom/vitest'

// jsdom does not implement IntersectionObserver; framer-motion's useInView
// needs it for CoverArt. The stub keeps tests deterministic (never in view,
// so no cover network calls are enqueued).
class IntersectionObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() {
    return []
  }
}

if (!('IntersectionObserver' in globalThis)) {
  Object.defineProperty(globalThis, 'IntersectionObserver', {
    writable: true,
    configurable: true,
    value: IntersectionObserverStub,
  })
}
