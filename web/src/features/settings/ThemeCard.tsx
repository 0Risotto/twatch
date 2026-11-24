import { Check } from 'lucide-react'
import { cn } from '@/lib/cn'
import type { ThemeMode } from '@/types/api'

export function ThemeCard({
  base,
  label,
  emoji,
  mode,
  active,
  onSelect,
}: {
  base: string
  label: string
  emoji: string
  mode: ThemeMode
  active: boolean
  onSelect: () => void
}) {
  return (
    <button
      type="button"
      aria-pressed={active}
      data-theme={`${base}-${mode}`}
      data-mode={mode}
      onClick={onSelect}
      className={cn(
        'group relative overflow-hidden rounded-[4px] border p-3 text-left transition-all',
        active
          ? 'border-[var(--accent)] shadow-[0_0_0_3px_var(--accent-subtle)]'
          : 'border-[var(--border)] hover:-translate-y-0.5 hover:border-[var(--border-strong)]',
      )}
      style={{ background: 'var(--bg)', color: 'var(--text)' }}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="flex min-w-0 items-center gap-1.5 truncate text-xs font-semibold">
          <span aria-hidden>{emoji}</span>
          <span className="truncate">{label}</span>
        </span>
        {active && (
          <span className="grid size-5 shrink-0 place-items-center rounded-full bg-[var(--accent)]">
            <Check className="size-3 text-[var(--accent-fg)]" />
          </span>
        )}
      </div>
      <div className="mt-3 flex gap-1.5">
        {[
          'var(--card)',
          'var(--accent)',
          'var(--ok)',
          'var(--warn)',
          'var(--danger)',
          'var(--info)',
        ].map((color, index) => (
          <span key={index} className="h-5 flex-1 rounded-[1px]" style={{ background: color }} />
        ))}
      </div>
      <div className="mt-2 flex items-center justify-between text-[9px] uppercase tracking-wider text-[var(--muted)]">
        <span>{mode}</span>
        <span className="font-mono">{base}</span>
      </div>
    </button>
  )
}
