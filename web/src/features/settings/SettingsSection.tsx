import { useState } from 'react'

import { ChevronDown } from 'lucide-react'

import { cn } from '@/lib/cn'

export function SettingsSection({
  id,
  icon,
  title,
  description,
  children,
  action,
  collapsible = false,
  defaultOpen = true,
}: {
  id?: string
  icon: React.ReactNode
  title: string
  description: string
  children: React.ReactNode
  action?: React.ReactNode
  collapsible?: boolean
  defaultOpen?: boolean
}) {
  const [open, setOpen] = useState(defaultOpen)
  const bodyId = id ? `${id}-body` : undefined
  const iconBadge = (
    <span className="grid size-9 shrink-0 place-items-center rounded-[3px] border border-[var(--border-soft)] bg-[var(--surface-2)] text-[var(--accent)]">
      {icon}
    </span>
  )

  return (
    <section id={id} className="panel scroll-mt-20 overflow-hidden">
      <header className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4 py-3.5">
        {collapsible ? (
          <button
            type="button"
            aria-expanded={open}
            aria-controls={bodyId}
            onClick={() => setOpen(value => !value)}
            className="flex min-w-0 flex-1 items-center gap-3 text-left"
          >
            {iconBadge}
            <span className="min-w-0">
              <span className="block text-sm font-semibold text-[var(--fg)]">{title}</span>
              <span className="mt-0.5 block text-[11px] text-[var(--muted)]">{description}</span>
            </span>
            <ChevronDown
              className={cn(
                'ml-auto size-4 shrink-0 text-[var(--muted)] transition-transform duration-200',
                open && 'rotate-180',
              )}
            />
          </button>
        ) : (
          <div className="flex items-center gap-3">
            {iconBadge}
            <div>
              <h2 className="text-sm font-semibold text-[var(--fg)]">{title}</h2>
              <p className="mt-0.5 text-[11px] text-[var(--muted)]">{description}</p>
            </div>
          </div>
        )}
        {action}
      </header>
      {(!collapsible || open) && (
        <div id={bodyId} className="p-4">
          {children}
        </div>
      )}
    </section>
  )
}
