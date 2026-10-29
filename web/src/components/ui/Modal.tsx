import { AnimatePresence, motion } from 'motion/react'
import * as Dialog from '@radix-ui/react-dialog'
import { X } from 'lucide-react'
import { cn } from '@/lib/cn'
import { fade, modalPanel } from '@/lib/motion'
import { IconButton } from './Button'

interface ModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description?: string
  children: React.ReactNode
  footer?: React.ReactNode
  maxWidth?: number
  className?: string
}

export function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
  footer,
  maxWidth = 560,
  className,
}: ModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <AnimatePresence>
        {open && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild forceMount>
              <motion.div
                variants={fade}
                initial="initial"
                animate="animate"
                exit="exit"
                className="fixed inset-0 z-50 bg-[color-mix(in_srgb,var(--bg)_62%,transparent)] backdrop-blur-[3px]"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild forceMount aria-describedby={description ? undefined : undefined}>
              <motion.div
                variants={modalPanel}
                initial="initial"
                animate="animate"
                exit="exit"
                className={cn(
                  'glass-strong fixed left-1/2 top-1/2 z-50 w-[calc(100vw-2rem)] -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-[var(--radius-xl)]',
                  className,
                )}
                style={{ maxWidth }}
              >
                <div className="flex items-start justify-between gap-4 border-b border-[var(--border-soft)] px-5 py-4">
                  <div className="min-w-0">
                    <Dialog.Title className="break-words pr-1 text-[15px] font-semibold tracking-[-0.01em] text-[var(--fg)]">
                      {title}
                    </Dialog.Title>
                    {description && (
                      <Dialog.Description className="mt-1 text-xs leading-relaxed text-[var(--muted)]">
                        {description}
                      </Dialog.Description>
                    )}
                  </div>
                  <Dialog.Close asChild>
                    <IconButton label="Close" size="sm">
                      <X className="size-4" />
                    </IconButton>
                  </Dialog.Close>
                </div>
                <div className="max-h-[min(70vh,640px)] overflow-y-auto px-5 py-4">{children}</div>
                {footer && (
                  <div className="flex items-center justify-end gap-2 border-t border-[var(--border-soft)] px-5 py-3">
                    {footer}
                  </div>
                )}
              </motion.div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  )
}
