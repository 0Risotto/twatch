import { useEffect, useRef, useState } from 'react'
import { useInView } from 'motion/react'
import { MediaIcon } from '@/lib/fileIcons'
import { useCover } from '@/lib/covers'
import { cn } from '@/lib/cn'

interface CoverBackdropProps {
  title: string
  /** Fade toward the left for horizontal cards, or upward for poster cards. */
  direction?: 'left' | 'bottom'
  className?: string
}

/**
 * Full-bleed artwork for media cards. The gradient sits over the image so text
 * stays readable while the cover remains visible on the right or at the top.
 */
export function CoverBackdrop({ title, direction = 'left', className }: CoverBackdropProps) {
  const ref = useRef<HTMLDivElement>(null)
  const inView = useInView(ref, { once: true, margin: '240px' })
  const cover = useCover(title, undefined, inView)
  const [failed, setFailed] = useState(false)

  useEffect(() => {
    setFailed(false)
  }, [cover?.url])

  const overlay =
    direction === 'bottom'
      ? 'linear-gradient(0deg, var(--card) 0%, color-mix(in srgb, var(--card) 82%, transparent) 48%, transparent 100%)'
      : 'linear-gradient(90deg, var(--card) 0%, color-mix(in srgb, var(--card) 88%, transparent) 52%, color-mix(in srgb, var(--card) 44%, transparent) 100%)'

  return (
    <div
      ref={ref}
      aria-hidden
      className={cn(
        'pointer-events-none absolute inset-0 overflow-hidden bg-[var(--surface-2)]',
        className,
      )}
    >
      {cover?.url && !failed ? (
        <img
          src={cover.url}
          alt=""
          loading="lazy"
          decoding="async"
          referrerPolicy="no-referrer"
          className="h-full w-full object-cover opacity-80 transition-transform duration-500 group-hover:scale-[1.03]"
          onError={() => setFailed(true)}
        />
      ) : (
        <div className="absolute -right-2 bottom-[-12%] text-[var(--border-strong)] opacity-50">
          <MediaIcon name={title} className="size-28" />
        </div>
      )}
      <div className="absolute inset-0" style={{ background: overlay }} />
    </div>
  )
}
