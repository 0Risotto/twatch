import { Modal } from '@/components/ui/Modal'
import { useUiStore } from '@/store/ui'

const shortcuts: Array<[string, string]> = [
  ['⌘K / Ctrl K', 'Command palette'],
  ['g then h', 'Home'],
  ['g then s', 'Search'],
  ['g then d', 'Downloads'],
  ['g then p', 'Player'],
  ['g then y', 'History'],
  ['g then t', 'Settings'],
  ['?', 'This help'],
  ['Esc', 'Close dialog / palette'],
]

export function ShortcutsHelp() {
  const open = useUiStore(state => state.shortcutsOpen)
  const setOpen = useUiStore(state => state.setShortcutsOpen)

  return (
    <Modal
      open={open}
      onOpenChange={setOpen}
      title="Keyboard shortcuts"
      description="twatch is keyboard-first. These work anywhere that is not an input."
      maxWidth={520}
    >
      <dl className="divide-y divide-[var(--border-soft)]">
        {shortcuts.map(([keys, label]) => (
          <div key={keys} className="flex items-center justify-between gap-4 py-2.5">
            <dt className="text-sm text-[var(--fg)]">{label}</dt>
            <dd className="kbd whitespace-nowrap">{keys}</dd>
          </div>
        ))}
      </dl>
    </Modal>
  )
}
