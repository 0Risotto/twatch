import { useEffect, useRef, useState } from 'react'
import { useInView } from 'motion/react'
import { cn } from '@/lib/cn'
import { FileTypeIcon, MediaIcon } from '@/lib/fileIcons'
import { useCover } from '@/lib/covers'

interface CoverArtProps {
  title: string
  kind?: string
  className?: string
  file?: boolean
  /** For grids that need a stable aspect box even before the image lands. */
  eager?: boolean
}

/**
 * A poster thumbnail backed by the batched cover service.  The fallback is the
 * same file-type icon used elsewhere, so a missing provider never leaves a hole.
 * Images are loaded near the viewport, not all at once for a 100-row search.
 */
export function CoverArt({ title, kind, className, file = false, eager = false }: CoverArtProps) {
  const ref = useRef<HTMLDivElement>(null)
  const inView = useInView(ref, { once: true, margin: '240px' })
  const cover = useCover(title, kind, eager || inView)
  const [failed, setFailed] = useState(false)

  useEffect(() => {
    setFailed(false)
  }, [cover?.url])

  return (
    <div
      ref={ref}
      className={cn(
        'relative overflow-hidden rounded-[7px] border border-[var(--border)] bg-[var(--surface-2)]',
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
          className="h-full w-full object-cover"
          onError={() => setFailed(true)}
        />
      ) : (
        <div className="grid h-full w-full place-items-center">
          {file ? (
            <FileTypeIcon name={title} className="size-1/2 opacity-80" />
          ) : (
            <MediaIcon name={title} className="size-1/2 opacity-80" />
          )}
        </div>
      )}
    </div>
  )
}
