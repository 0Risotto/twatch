import type { ReactNode } from 'react'
import { motion } from 'motion/react'
import { easeOut } from '@/lib/motion'

interface PageHeaderProps {
  eyebrow?: string
  title: string
  description?: string
  actions?: ReactNode
  media?: ReactNode
}

export function PageHeader({ eyebrow, title, description, actions, media }: PageHeaderProps) {
  return (
    <motion.header
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.24, ease: easeOut }}
      className="mb-5 flex flex-wrap items-start justify-between gap-4"
    >
      <div className="flex min-w-0 items-start gap-4">
        {media}
        <div className="min-w-0">
          {eyebrow && (
            <p className="mb-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--accent)]">
              {eyebrow}
            </p>
          )}
          <h1 className="text-[clamp(1.55rem,2.6vw,2.2rem)] font-semibold leading-tight tracking-[-0.035em] text-[var(--fg)]">
            {title}
          </h1>
          {description && (
            <p className="mt-1.5 max-w-2xl text-sm leading-relaxed text-[var(--muted)]">{description}</p>
          )}
        </div>
      </div>
      {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
    </motion.header>
  )
}

export function SectionHeader({
  title,
  hint,
  action,
}: {
  title: string
  hint?: string
  action?: ReactNode
}) {
  return (
    <div className="mb-3 flex items-center justify-between gap-3">
      <div>
        <h2 className="text-sm font-semibold tracking-[-0.01em] text-[var(--fg)]">{title}</h2>
        {hint && <p className="mt-0.5 text-[11px] text-[var(--muted)]">{hint}</p>}
      </div>
      {action}
    </div>
  )
}
