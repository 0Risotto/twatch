import { useEffect } from 'react'
import { motion, useSpring, useTransform } from 'motion/react'
import { cn } from '@/lib/cn'

interface ProgressBarProps {
  progress: number
  className?: string
  tone?: 'accent' | 'success' | 'warning' | 'stream'
  height?: number
}

const toneColors = {
  accent: 'linear-gradient(90deg, var(--accent), var(--badge-stream))',
  success: 'linear-gradient(90deg, var(--success), color-mix(in srgb, var(--success) 60%, var(--accent)))',
  warning: 'linear-gradient(90deg, var(--warning), var(--badge-dl))',
  stream: 'linear-gradient(90deg, var(--badge-stream), var(--accent))',
}

export function ProgressBar({ progress, className, tone = 'accent', height = 6 }: ProgressBarProps) {
  const value = useSpring(0, { stiffness: 120, damping: 24, mass: 0.9 })
  const scaleX = useTransform(value, latest => Math.max(0, Math.min(1, latest)))

  useEffect(() => {
    value.set(progress)
  }, [progress, value])

  return (
    <div
      className={cn('relative w-full overflow-hidden rounded-full bg-[var(--surface-3)]', className)}
      style={{ height }}
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(progress * 100)}
    >
      <motion.div
        className="absolute inset-y-0 left-0 w-full origin-left rounded-full"
        style={{ scaleX, background: toneColors[tone] }}
      />
      <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(255,255,255,0.16),transparent_55%)]" />
    </div>
  )
}

interface ProgressRingProps {
  progress: number
  size?: number
  stroke?: number
  className?: string
  children?: React.ReactNode
}

export function ProgressRing({ progress, size = 56, stroke = 5, className, children }: ProgressRingProps) {
  const radius = (size - stroke) / 2
  const circumference = 2 * Math.PI * radius
  const value = useSpring(0, { stiffness: 110, damping: 22, mass: 0.9 })
  const dash = useTransform(value, latest => circumference * (1 - Math.max(0, Math.min(1, latest))))

  useEffect(() => {
    value.set(progress)
  }, [progress, value])

  return (
    <div className={cn('relative inline-grid place-items-center', className)} style={{ width: size, height: size }}>
      <svg width={size} height={size} className="-rotate-90" aria-hidden>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--surface-3)"
          strokeWidth={stroke}
        />
        <motion.circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--accent)"
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          style={{ strokeDashoffset: dash }}
        />
      </svg>
      <div className="absolute inset-0 grid place-items-center">{children}</div>
    </div>
  )
}

interface SparklineProps {
  values: number[]
  width?: number
  height?: number
  className?: string
}

export function Sparkline({ values, width = 120, height = 32, className }: SparklineProps) {
  const max = Math.max(1, ...values)
  const points = values.length
    ? values
        .map((value, index) => {
          const x = (index / Math.max(1, values.length - 1)) * width
          const y = height - (value / max) * (height - 4) - 2
          return `${x.toFixed(1)},${y.toFixed(1)}`
        })
        .join(' ')
    : `0,${height - 2} ${width},${height - 2}`

  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} className={className} aria-hidden>
      <polyline
        points={`0,${height} ${points} ${width},${height}`}
        fill="color-mix(in srgb, var(--accent) 10%, transparent)"
        stroke="none"
      />
      <polyline
        points={points}
        fill="none"
        stroke="var(--accent)"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
