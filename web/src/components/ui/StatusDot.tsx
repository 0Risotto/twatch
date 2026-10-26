import { cn } from '@/lib/cn'

type Tone = 'idle' | 'active' | 'success' | 'warning' | 'danger' | 'stream'

const tones: Record<Tone, string> = {
  idle: 'bg-[var(--muted)]',
  active: 'bg-[var(--accent)]',
  success: 'bg-[var(--success)]',
  warning: 'bg-[var(--warning)]',
  danger: 'bg-[var(--danger)]',
  stream: 'bg-[var(--badge-stream)]',
}

export function StatusDot({ tone = 'idle', pulse = false, className }: { tone?: Tone; pulse?: boolean; className?: string }) {
  return (
    <span className={cn('relative flex size-2 shrink-0', className)} aria-hidden>
      {pulse && <span className={cn('absolute inset-0 animate-ping rounded-full opacity-50', tones[tone])} />}
      <span className={cn('relative size-2 rounded-full', tones[tone])} />
    </span>
  )
}
