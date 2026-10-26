import { Link } from 'react-router-dom'
import { Compass, Home } from 'lucide-react'
import { EmptyState } from '@/components/ui/EmptyState'

export function NotFoundPage() {
  return (
    <div className="grid min-h-[60vh] place-items-center">
      <EmptyState
        icon={<Compass className="size-6" />}
        title="That route is not on the map"
        description="The page may have moved, or the link is from an older build."
        action={
          <Link to="/" className="btn btn-primary">
            <Home className="size-4" />
            Back home
          </Link>
        }
      />
    </div>
  )
}
