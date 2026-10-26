import { useId } from 'react'
import { motion } from 'motion/react'
import { cn } from '@/lib/cn'
import { springs } from '@/lib/motion'

interface Option<T extends string> {
  value: T
  label: string
  icon?: React.ReactNode
}

interface SegmentedControlProps<T extends string> {
  options: Option<T>[]
  value: T
  onChange: (value: T) => void
  className?: string
}

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  className,
}: SegmentedControlProps<T>) {
  const id = useId()
  return (
    <div
      className={cn(
        'relative inline-flex items-center gap-0.5 rounded-[10px] border border-[var(--border-soft)] bg-[var(--surface-2)] p-0.5',
        className,
      )}
      role="tablist"
    >
      {options.map(option => {
        const active = option.value === value
        return (
          <button
            key={option.value}
            type="button"
            role="tab"
            aria-selected={active}
            onClick={() => onChange(option.value)}
            className={cn(
              'relative z-10 inline-flex h-7 items-center gap-1.5 rounded-[8px] px-3 text-xs font-semibold transition-colors',
              active ? 'text-[var(--fg)]' : 'text-[var(--muted)] hover:text-[var(--fg)]',
            )}
          >
            {active && (
              <motion.span
                layoutId={`segment-${id}`}
                className="absolute inset-0 -z-10 rounded-[8px] bg-[var(--surface)] shadow-[var(--shadow-sm)]"
                transition={springs.layout}
              />
            )}
            {option.icon}
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
