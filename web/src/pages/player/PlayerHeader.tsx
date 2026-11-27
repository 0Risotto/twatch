import { Link } from 'react-router-dom'
import { ArrowLeft, Eye } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/PageHeader'
import { formatBytes } from '@/lib/format'
import type { DownloadSnapshot } from '@/types/api'

interface PlayerHeaderProps {
  download: DownloadSnapshot
  streamUrl: string | null
  previewable: boolean
  onBack: () => void
}

export function PlayerHeader({ download, streamUrl, previewable, onBack }: PlayerHeaderProps) {
  return (
    <PageHeader
      eyebrow="Playback"
      title={download.label}
      description={`${download.torrentName} · ${formatBytes(download.totalSize)}`}
      actions={
        <>
          <Button variant="ghost" size="sm" onClick={onBack}>
            <ArrowLeft className="size-4" />
            Back
          </Button>
          <Link to="/downloads" className="btn btn-ghost h-8 px-3 text-xs">
            Downloads
          </Link>
          {streamUrl && !previewable && (
            <a
              href={streamUrl}
              target="_blank"
              rel="noreferrer"
              className="btn btn-secondary h-8 px-3 text-xs"
            >
              <Eye className="size-4" />
              Open file
            </a>
          )}
        </>
      }
    />
  )
}
