import { useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { motion } from 'motion/react'
import {
  ArrowDown,
  HardDrive,
  Loader2,
  Magnet,
  Search,
  Settings2,
  Sparkles,
  Upload,
  Users,
} from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { SegmentedControl } from '@/components/ui/SegmentedControl'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { formatBytes, magnetFor } from '@/lib/format'
import { CoverArt } from '@/components/CoverArt'
import { riseItem, stagger } from '@/lib/motion'
import { useDebounced } from '@/lib/useDebounced'
import { useAppStore } from '@/store/app'
import { cn } from '@/lib/cn'
import type { SearchResult } from '@/types/api'

type Sort = 'seeders' | 'size'

export function SearchPage() {
  const navigate = useNavigate()
  const searchConfigured = useAppStore(state => state.searchConfigured)
  const setSession = useAppStore(state => state.setSession)
  const addToast = useAppStore(state => state.addToast)

  const [searchParams] = useSearchParams()
  const queryParam = searchParams.get('q')
  const [query, setQuery] = useState(queryParam ?? '')
  const [sort, setSort] = useState<Sort>('seeders')
  const [activeIndex, setActiveIndex] = useState(0)
  const [opening, setOpening] = useState<string | null>(null)
  const debounced = useDebounced(query.trim(), 280)

  const search = useQuery({
    queryKey: ['search', debounced],
    queryFn: () => api.search(debounced, 0),
    enabled: debounced.length > 0 && searchConfigured,
    placeholderData: previous => previous,
    staleTime: 30_000,
  })

  useEffect(() => {
    if (queryParam !== null && queryParam !== query) {
      setQuery(queryParam)
      setActiveIndex(0)
    }
    // Only react to URL changes, not to local typing.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [queryParam])

  const results = useMemo(() => {
    const items = [...(search.data ?? [])]
    items.sort((a, b) => (sort === 'seeders' ? b.seeders - a.seeders : b.size - a.size))
    return items
  }, [search.data, sort])

  const open = async (result: SearchResult) => {
    if (opening) return
    setOpening(result.infoHash)
    try {
      const session = await api.preview(magnetFor(result))
      setSession(session)
      addToast('success', `${session.name} — ${session.files.length} files ready`)
      navigate('/torrent')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not load torrent')
    } finally {
      setOpening(null)
    }
  }

  const onKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      setActiveIndex(index => Math.min(results.length - 1, index + 1))
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      setActiveIndex(index => Math.max(0, index - 1))
    } else if (event.key === 'Enter' && results[activeIndex]) {
      event.preventDefault()
      void open(results[activeIndex])
    }
  }

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Discovery"
        title="Find something worth watching"
        description="Search the JSON API you configured. Query text and poster titles go to the search and cover providers; your files stay local."
        actions={
          <Button variant="secondary" size="sm" onClick={() => navigate('/settings')}>
            <Settings2 className="size-4" />
            Search API
          </Button>
        }
      />

      <div className="panel p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-[var(--muted)]" />
          {search.isFetching && (
            <Loader2 className="absolute right-4 top-1/2 size-4 -translate-y-1/2 animate-spin text-[var(--accent)]" />
          )}
          <input
            value={query}
            onChange={event => {
              setQuery(event.target.value)
              setActiveIndex(0)
            }}
            onKeyDown={onKeyDown}
            placeholder="Search releases, titles, uploaders…"
            aria-label="Search torrents"
            autoFocus
            className="h-12 w-full rounded-[12px] border border-[var(--border-soft)] bg-[var(--surface-2)] pl-11 pr-12 text-sm text-[var(--fg)] outline-none transition placeholder:text-[var(--faint)] focus:border-[color-mix(in_srgb,var(--accent)_52%,transparent)] focus:shadow-[0_0_0_3px_var(--accent-soft)]"
          />
        </div>
        <div className="mt-3 flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2 text-[11px] text-[var(--muted)]">
            <span className="kbd">↑↓</span> navigate
            <span className="kbd ml-2">↵</span> preview
            <span className="ml-2 hidden sm:inline">· {results.length} results</span>
          </div>
          <SegmentedControl
            options={[
              { value: 'seeders', label: 'Top seeded' },
              { value: 'size', label: 'Largest' },
            ]}
            value={sort}
            onChange={value => setSort(value as Sort)}
          />
        </div>
      </div>

      {!searchConfigured && (
        <EmptyState
          icon={<Settings2 className="size-6" />}
          title="Connect a search API"
          description="twatch does not bundle a torrent index. Point it at any JSON endpoint that returns an array; map the fields once in Settings."
          action={
            <Button variant="primary" onClick={() => navigate('/settings')}>
              Configure search
            </Button>
          }
        />
      )}

      {searchConfigured && !debounced && (
        <EmptyState
          icon={<Sparkles className="size-6" />}
          title="Search the index"
          description="Type at least two characters. Results are ranked by seeders by default and can be previewed before anything downloads."
        />
      )}

      {searchConfigured && debounced && search.isLoading && <SkeletonRows count={6} />}

      {searchConfigured && debounced && search.isError && (
        <EmptyState
          icon={<Search className="size-6" />}
          title="Search failed"
          description={search.error instanceof Error ? search.error.message : 'The search API did not respond.'}
          action={
            <Button variant="secondary" onClick={() => void search.refetch()}>
              Try again
            </Button>
          }
        />
      )}

      {searchConfigured && debounced && !search.isLoading && !search.isError && results.length === 0 && (
        <EmptyState
          icon={<Magnet className="size-6" />}
          title="No results"
          description="Nothing matched that query. Try a shorter title or a different uploader."
        />
      )}

      {results.length > 0 && (
        <motion.div variants={stagger} initial="initial" animate="animate" className="space-y-2">
          {results.map((result, index) => (
            <motion.div
              key={result.infoHash}
              variants={riseItem}
              className={cn(
                'group flex items-center gap-3 rounded-[13px] border bg-[color-mix(in_srgb,var(--surface)_86%,transparent)] px-3 py-2.5 transition-colors',
                index === activeIndex
                  ? 'border-[color-mix(in_srgb,var(--accent)_38%,transparent)] bg-[color-mix(in_srgb,var(--accent)_6%,var(--surface))]'
                  : 'border-[var(--border-soft)] hover:border-[var(--border-strong)]',
              )}
              onMouseEnter={() => setActiveIndex(index)}
            >
              <CoverArt title={result.name} className="h-16 w-11 shrink-0" />
              <div className="min-w-0 flex-1">
                <p className="truncate text-[13px] font-medium text-[var(--fg)]">{result.name}</p>
                <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[10px] text-[var(--muted)]">
                  <span className="flex items-center gap-1 text-[var(--success)]">
                    <ArrowDown className="size-3" />
                    {result.seeders.toLocaleString()}
                  </span>
                  <span className="flex items-center gap-1">
                    <Users className="size-3" />
                    {result.leechers.toLocaleString()}
                  </span>
                  <span className="flex items-center gap-1">
                    <HardDrive className="size-3" />
                    {formatBytes(result.size)}
                  </span>
                  {result.username && (
                    <span className="flex items-center gap-1">
                      <Upload className="size-3" />
                      {result.username}
                    </span>
                  )}
                </div>
              </div>
              <Button
                variant={index === activeIndex ? 'primary' : 'secondary'}
                size="sm"
                loading={opening === result.infoHash}
                onClick={() => void open(result)}
                className="shrink-0"
              >
                Preview
              </Button>
            </motion.div>
          ))}
        </motion.div>
      )}

      {results.length > 0 && (
        <p className="pb-2 text-center text-[10px] text-[var(--faint)]">
          Results come from your configured API. twatch never proxies or stores the index.
        </p>
      )}
    </div>
  )
}
