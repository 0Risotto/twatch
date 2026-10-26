import { useEffect } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { AlertTriangle, CheckCircle2, Info, X, XCircle } from 'lucide-react'
import { useAppStore } from '@/store/app'
import { springs } from '@/lib/motion'
import type { ToastItem, ToastLevel } from '@/types/api'

const icons: Record<ToastLevel, typeof Info> = {
  info: Info,
  success: CheckCircle2,
  warning: AlertTriangle,
  error: XCircle,
}

const accents: Record<ToastLevel, string> = {
  info: 'var(--badge-stream)',
  success: 'var(--success)',
  warning: 'var(--warning)',
  error: 'var(--danger)',
}

function Toast({ toast }: { toast: ToastItem }) {
  const dismissToast = useAppStore(state => state.dismissToast)
  const Icon = icons[toast.level]

  useEffect(() => {
    const timer = window.setTimeout(() => dismissToast(toast.id), toast.level === 'error' ? 6500 : 4200)
    return () => window.clearTimeout(timer)
  }, [dismissToast, toast.id, toast.level])

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 14, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: 8, scale: 0.98, transition: { duration: 0.12 } }}
      transition={springs.snappy}
      className="glass-strong pointer-events-auto flex w-[min(360px,calc(100vw-2rem))] items-start gap-3 rounded-[var(--radius-lg)] border-l-2 p-3.5"
      style={{ borderLeftColor: accents[toast.level] }}
    >
      <Icon className="mt-0.5 size-4 shrink-0" style={{ color: accents[toast.level] }} aria-hidden />
      <p className="min-w-0 flex-1 text-[13px] leading-snug text-[var(--fg)]">{toast.message}</p>
      <button
        type="button"
        onClick={() => dismissToast(toast.id)}
        className="-m-1 rounded-md p-1 text-[var(--muted)] transition hover:bg-[var(--hover)] hover:text-[var(--fg)]"
        aria-label="Dismiss notification"
      >
        <X className="size-3.5" />
      </button>
    </motion.div>
  )
}

export function ToastViewport() {
  const toasts = useAppStore(state => state.toasts)
  return (
    <div className="pointer-events-none fixed bottom-4 right-4 z-[70] flex flex-col items-end gap-2">
      <AnimatePresence initial={false}>
        {toasts.map(toast => (
          <Toast key={toast.id} toast={toast} />
        ))}
      </AnimatePresence>
    </div>
  )
}
