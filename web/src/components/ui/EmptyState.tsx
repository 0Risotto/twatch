import type { ReactNode } from 'react'
import { motion } from 'motion/react'
import { cn } from '@/lib/cn'
import { fade } from '@/lib/motion'

interface EmptyStateProps {
  icon: ReactNode
  title: string
  description?: string
  action?: ReactNode
  className?: string
}

export function EmptyState({ icon, title, description, action, className }: EmptyStateProps) {
  return (
    <motion.div
      variants={fade}
      initial="initial"
      animate="animate"
      className={cn(
        'flex min-h-[280px] flex-col items-center justify-center rounded-[var(--radius-xl)] border border-dashed border-[var(--border-strong)] bg-[color-mix(in_srgb,var(--surface)_40%,transparent)] px-6 py-12 text-center',
        className,
      )}
    >
      <div className="mb-4 grid size-14 place-items-center rounded-2xl border border-[var(--border-soft)] bg-[var(--surface-2)] text-[var(--accent)] shadow-[var(--shadow-sm)]">
        {icon}
      </div>
      <h3 className="text-base font-semibold tracking-[-0.01em] text-[var(--fg)]">{title}</h3>
      {description && (
        <p className="mt-1.5 max-w-sm text-sm leading-relaxed text-[var(--muted)]">{description}</p>
      )}
      {action && <div className="mt-5">{action}</div>}
    </motion.div>
  )
}
