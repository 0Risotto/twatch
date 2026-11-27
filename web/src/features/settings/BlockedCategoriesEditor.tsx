import { useState } from 'react'
import { X } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { TextInput } from '@/components/ui/Field'

interface BlockedCategoriesEditorProps {
  value: string[]
  onChange: (value: string[]) => void
}

export function BlockedCategoriesEditor({ value, onChange }: BlockedCategoriesEditorProps) {
  const [draft, setDraft] = useState('')

  const add = () => {
    const next = draft.trim()
    if (!next) return
    if (!value.some(item => item.toLowerCase() === next.toLowerCase())) {
      onChange([...value, next])
    }
    setDraft('')
  }

  return (
    <div className="space-y-2">
      {value.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {value.map(item => (
            <span
              key={item}
              className="inline-flex items-center gap-1 rounded-[2px] border border-[var(--border-soft)] bg-[var(--surface-2)] px-2 py-1 font-mono text-[11px] text-[var(--fg)]"
            >
              {item}
              <button
                type="button"
                aria-label={`Remove ${item}`}
                onClick={() => onChange(value.filter(entry => entry !== item))}
                className="text-[var(--muted)] transition hover:text-[var(--danger)]"
              >
                <X className="size-3" />
              </button>
            </span>
          ))}
        </div>
      )}
      <div className="flex items-center gap-2">
        <TextInput
          aria-label="Add blocked category"
          value={draft}
          onChange={event => setDraft(event.target.value)}
          onKeyDown={event => {
            if (event.key === 'Enter') {
              event.preventDefault()
              add()
            }
          }}
          placeholder="Category value, range, or word"
          className="font-mono text-xs"
        />
        <Button type="button" variant="secondary" size="sm" onClick={add} disabled={!draft.trim()}>
          Add
        </Button>
      </div>
    </div>
  )
}
