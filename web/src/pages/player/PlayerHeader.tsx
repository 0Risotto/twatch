import { Link } from 'react-router-dom'

import { ArrowLeft, Eye } from 'lucide-react'

import { formatBytes } from '@/lib/format'

import { Button } from '@/components/ui/Button'

import { PageHeader } from '@/components/PageHeader'

import type { PlaybackTrack } from '@/features/player/playbackTrack'

interface PlayerHeaderProps {
  track: PlaybackTrack
  streamUrl: string | null
  previewable: boolean
  onBack: () => void
}

export function PlayerHeader({ track, streamUrl, previewable, onBack }: PlayerHeaderProps) {
  return (
    <PageHeader
      eyebrow="Playback"
      title={track.title}
      description={`${track.subtitle} · ${formatBytes(track.size)}`}
      actions={
        <>
          <Button variant="ghost" size="sm" onClick={onBack}>
            <ArrowLeft className="size-4" />
            Back
          </Button>
          <Link
            to={track.source === 'library' ? '/library' : '/downloads'}
            className="btn btn-ghost h-8 px-3 text-xs"
          >
            {track.source === 'library' ? 'Library' : 'Downloads'}
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
