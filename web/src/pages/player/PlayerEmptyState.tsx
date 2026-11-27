import { Link } from 'react-router-dom'
import { MonitorPlay, Play } from 'lucide-react'
import { EmptyState } from '@/components/ui/EmptyState'
import { PageHeader } from '@/components/PageHeader'

export function PlayerEmptyState() {
  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Playback"
        title="Now playing"
        description="Inline playback appears here while the engine streams pieces from the swarm."
        actions={
          <Link to="/downloads" className="btn btn-secondary h-8 px-3 text-xs">
            Downloads
          </Link>
        }
      />
      <EmptyState
        icon={<MonitorPlay className="size-6" />}
        title="Nothing playing"
        description="Pick a file in a torrent and press Watch. twatch starts a one-file stream and keeps downloading in the background."
        action={
          <Link to="/search" className="btn btn-secondary">
            Find a torrent
            <Play className="size-4" />
          </Link>
        }
      />
    </div>
  )
}
