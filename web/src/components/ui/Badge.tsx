import type { ReactNode } from 'react'
import { cn } from '@/lib/cn'

type BadgeTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'stream' | 'download' | 'folder'

const tones: Record<BadgeTone, string> = {
  neutral: 'bg-[var(--surface-2)] text-[var(--muted-strong)] border-[var(--border-soft)]',
  accent: 'bg-[var(--accent-soft)] text-[var(--accent)] border-[color-mix(in_srgb,var(--accent)_28%,transparent)]',
  success: 'bg-[var(--success-soft)] text-[var(--success)] border-[color-mix(in_srgb,var(--success)_28%,transparent)]',
  warning: 'bg-[var(--warning-soft)] text-[var(--warning)] border-[color-mix(in_srgb,var(--warning)_28%,transparent)]',
  danger: 'bg-[var(--danger-soft)] text-[var(--danger)] border-[color-mix(in_srgb,var(--danger)_28%,transparent)]',
  stream: 'bg-[var(--stream-soft)] text-[var(--badge-stream)] border-[color-mix(in_srgb,var(--badge-stream)_28%,transparent)]',
  download: 'bg-[var(--warning-soft)] text-[var(--badge-dl)] border-[color-mix(in_srgb,var(--badge-dl)_28%,transparent)]',
  folder: 'bg-[color-mix(in_srgb,var(--folder)_14%,transparent)] text-[var(--folder)] border-[color-mix(in_srgb,var(--folder)_26%,transparent)]',
}

interface BadgeProps {
  children: ReactNode
  tone?: BadgeTone
  dot?: boolean
  className?: string
}

export function Badge({ children, tone = 'neutral', dot = false, className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.08em]',
        tones[tone],
        className,
      )}
    >
      {dot && <span className="size-1.5 rounded-full bg-current" aria-hidden />}
      {children}
    </span>
  )
}
