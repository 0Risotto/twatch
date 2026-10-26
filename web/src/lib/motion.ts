import type { Transition, Variants } from 'motion/react'

export const easeOut = [0.16, 1, 0.3, 1] as const
export const easeInOut = [0.32, 0.72, 0, 1] as const
export const easeIn = [0.4, 0, 1, 1] as const

export const springs = {
  snappy: { type: 'spring', stiffness: 520, damping: 38, mass: 0.8 } satisfies Transition,
  smooth: { type: 'spring', stiffness: 320, damping: 30, mass: 0.9 } satisfies Transition,
  soft: { type: 'spring', stiffness: 260, damping: 26, mass: 1 } satisfies Transition,
  layout: { type: 'spring', stiffness: 380, damping: 34, mass: 0.9 } satisfies Transition,
}

export const durations = {
  instant: 0.08,
  fast: 0.14,
  base: 0.22,
  slow: 0.32,
} as const

export const pageVariants: Variants = {
  initial: { opacity: 0, y: 10 },
  animate: {
    opacity: 1,
    y: 0,
    transition: { duration: durations.base, ease: easeOut },
  },
  exit: {
    opacity: 0,
    y: -6,
    transition: { duration: durations.fast, ease: easeIn },
  },
}

export const modalPanel = {
  initial: { opacity: 0, scale: 0.97, y: 8 },
  animate: { opacity: 1, scale: 1, y: 0, transition: springs.snappy },
  exit: { opacity: 0, scale: 0.98, y: 4, transition: { duration: durations.fast, ease: easeIn } },
}

export const fade = {
  initial: { opacity: 0 },
  animate: { opacity: 1, transition: { duration: durations.base, ease: easeOut } },
  exit: { opacity: 0, transition: { duration: durations.fast, ease: easeIn } },
}

export const stagger = {
  animate: { transition: { staggerChildren: 0.03, delayChildren: 0.02 } },
  initial: {},
}

export const riseItem: Variants = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0, transition: { duration: 0.22, ease: easeOut } },
}
