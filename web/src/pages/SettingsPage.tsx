import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Check,
  Database,
  ExternalLink,
  Monitor,
  Palette,
  Play,
  RefreshCw,
  Search,
  Server,
  Sparkles,
  Volume2,
} from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Field, Label, Select, TextInput } from '@/components/ui/Field'
import { SegmentedControl } from '@/components/ui/SegmentedControl'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { applyTheme, splitTheme, themeLabel, themeName, THEME_BASES } from '@/lib/themes'
import { cn } from '@/lib/cn'
import { useAppStore } from '@/store/app'
import { useUiStore, type MotionPreference } from '@/store/ui'
import type { FieldMap, SearchConfig, ThemeMode } from '@/types/api'

function SettingsSection({
  icon,
  title,
  description,
  children,
  action,
}: {
  icon: React.ReactNode
  title: string
  description: string
  children: React.ReactNode
  action?: React.ReactNode
}) {
  return (
    <section className="panel overflow-hidden">
      <header className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4 py-3.5">
        <div className="flex items-center gap-3">
          <span className="grid size-9 place-items-center rounded-[10px] border border-[var(--border-soft)] bg-[var(--surface-2)] text-[var(--accent)]">
            {icon}
          </span>
          <div>
            <h2 className="text-sm font-semibold text-[var(--fg)]">{title}</h2>
            <p className="mt-0.5 text-[11px] text-[var(--muted)]">{description}</p>
          </div>
        </div>
        {action}
      </header>
      <div className="p-4">{children}</div>
    </section>
  )
}

function ThemeCard({
  base,
  label,
  emoji,
  mode,
  active,
  onSelect,
}: {
  base: string
  label: string
  emoji: string
  mode: ThemeMode
  active: boolean
  onSelect: () => void
}) {
  return (
    <button
      type="button"
      data-theme={`${base}-${mode}`}
      data-mode={mode}
      onClick={onSelect}
      className={cn(
        'group relative overflow-hidden rounded-[12px] border p-3 text-left transition-all',
        active
          ? 'border-[var(--accent)] shadow-[0_0_0_3px_var(--accent-subtle)]'
          : 'border-[var(--border)] hover:-translate-y-0.5 hover:border-[var(--border-strong)]',
      )}
      style={{ background: 'var(--bg)', color: 'var(--text)' }}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="flex min-w-0 items-center gap-1.5 truncate text-xs font-semibold">
          <span aria-hidden>{emoji}</span>
          <span className="truncate">{label}</span>
        </span>
        {active && (
          <span className="grid size-5 shrink-0 place-items-center rounded-full bg-[var(--accent)]">
            <Check className="size-3 text-[var(--accent-fg)]" />
          </span>
        )}
      </div>
      <div className="mt-3 flex gap-1.5">
        {['var(--card)', 'var(--accent)', 'var(--ok)', 'var(--warn)', 'var(--danger)', 'var(--info)'].map(
          (color, index) => (
            <span key={index} className="h-5 flex-1 rounded-[4px]" style={{ background: color }} />
          ),
        )}
      </div>
      <div className="mt-2 flex items-center justify-between text-[9px] uppercase tracking-wider text-[var(--muted)]">
        <span>{mode}</span>
        <span className="font-mono">{base}</span>
      </div>
    </button>
  )
}

const defaultFieldMap: FieldMap = {
  name: 'name',
  infoHash: 'info_hash',
  seeders: 'seeders',
  leechers: 'leechers',
  size: 'size',
  username: 'username',
}

export function SettingsPage() {
  const config = useAppStore(state => state.config)
  const bootstrap = useAppStore(state => state.bootstrap)
  const connected = useAppStore(state => state.connected)
  const setConfig = useAppStore(state => state.setConfig)
  const setSearchConfigured = useAppStore(state => state.setSearchConfigured)
  const addToast = useAppStore(state => state.addToast)
  const motion = useUiStore(state => state.motion)
  const setMotion = useUiStore(state => state.setMotion)

  const searchConfig = useQuery({ queryKey: ['searchConfig'], queryFn: api.getSearchConfig })
  const [searchUrl, setSearchUrl] = useState('')
  const [headersText, setHeadersText] = useState('{}')
  const [fieldMap, setFieldMap] = useState<FieldMap>(defaultFieldMap)
  const [savingSearch, setSavingSearch] = useState(false)
  const [testingSearch, setTestingSearch] = useState(false)
  const [savingTheme, setSavingTheme] = useState<string | null>(null)
  const [mode, setMode] = useState<ThemeMode>(() => splitTheme(config?.theme ?? 'emerald-dark').mode)
  const currentBase = splitTheme(config?.theme ?? 'emerald-dark').base

  useEffect(() => {
    if (config?.theme) setMode(splitTheme(config.theme).mode)
  }, [config?.theme])

  useEffect(() => {
    if (!searchConfig.data) return
    setSearchUrl(searchConfig.data.url)
    setHeadersText(JSON.stringify(searchConfig.data.headers, null, 2))
    setFieldMap({ ...defaultFieldMap, ...searchConfig.data.fieldMap })
  }, [searchConfig.data])

  const chooseTheme = async (base: string, nextMode: ThemeMode) => {
    const name = themeName(base, nextMode)
    applyTheme(name)
    setSavingTheme(name)
    try {
      const next = await api.patchConfig({ theme: name })
      setConfig(next)
      addToast('success', `Theme set to ${themeLabel(base)} ${nextMode}`)
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not save theme')
    } finally {
      setSavingTheme(null)
    }
  }

  const choosePlayer = async (player: string) => {
    try {
      const next = await api.patchConfig({ player })
      setConfig(next)
      addToast('success', `External player: ${player}`)
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not save player preference')
    }
  }

  const saveSearch = async () => {
    setSavingSearch(true)
    try {
      let headers: Record<string, string> = {}
      if (headersText.trim()) {
        const parsed = JSON.parse(headersText) as unknown
        if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
          throw new Error('Headers must be a JSON object')
        }
        headers = Object.fromEntries(
          Object.entries(parsed as Record<string, unknown>).map(([key, value]) => [key, String(value)]),
        )
      }
      const next: SearchConfig = { url: searchUrl.trim(), headers, fieldMap }
      await api.putSearchConfig(next)
      setSearchConfigured(Boolean(next.url))
      await searchConfig.refetch()
      addToast('success', next.url ? 'Search API saved' : 'Search API cleared')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not save search API')
    } finally {
      setSavingSearch(false)
    }
  }

  const testSearch = async () => {
    setTestingSearch(true)
    try {
      const results = await api.search('ubuntu', 0)
      addToast(
        results.length ? 'success' : 'warning',
        results.length ? `Search API returned ${results.length} results` : 'Search API responded with no results',
      )
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Search request failed')
    } finally {
      setTestingSearch(false)
    }
  }

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Preferences"
        title="Shape your console"
        description="Themes, motion, playback, and the JSON search endpoint. Everything is stored locally."
      />

      <SettingsSection
        icon={<Palette className="size-4" />}
        title="Appearance"
        description="Built-in palettes, each with matching dark and light tokens."
        action={
          savingTheme ? (
            <span className="text-[11px] text-[var(--muted)]">Saving…</span>
          ) : (
            <span className="text-[11px] text-[var(--muted)]">{config?.theme ?? 'emerald-dark'}</span>
          )
        }
      >
        <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
          <p className="text-xs text-[var(--muted)]">
            {THEME_BASES.length} built-in palettes · each has a dark and light variant
          </p>
          <SegmentedControl
            options={[
              { value: 'dark', label: 'Dark' },
              { value: 'light', label: 'Light' },
            ]}
            value={mode}
            onChange={value => {
              const nextMode = value as ThemeMode
              setMode(nextMode)
              void chooseTheme(currentBase, nextMode)
            }}
          />
        </div>
        <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {THEME_BASES.map(theme => (
            <ThemeCard
              key={theme.value}
              base={theme.value}
              label={theme.label}
              emoji={theme.emoji}
              mode={mode}
              active={config?.theme === `${theme.value}-${mode}`}
              onSelect={() => void chooseTheme(theme.value, mode)}
            />
          ))}
        </div>
      </SettingsSection>

      <div className="grid gap-4 lg:grid-cols-2">
        <SettingsSection
          icon={<Sparkles className="size-4" />}
          title="Motion"
          description="System respects prefers-reduced-motion. Reduced disables ambient and page motion."
        >
          <SegmentedControl
            options={[
              { value: 'system', label: 'System' },
              { value: 'full', label: 'Full' },
              { value: 'reduced', label: 'Reduced' },
            ]}
            value={motion}
            onChange={value => setMotion(value as MotionPreference)}
          />
          <p className="mt-3 text-[11px] leading-relaxed text-[var(--muted)]">
            Motion is intentionally short: 120–320 ms, springs for panels and selection, fades for pages.
            Active transfers pulse ambiently only while the tab is visible.
          </p>
        </SettingsSection>

        <SettingsSection
          icon={<Volume2 className="size-4" />}
          title="Playback"
          description="Inline video handles browser-friendly codecs. Everything else can go to mpv or VLC."
          action={<Play className="size-4 text-[var(--badge-stream)]" />}
        >
          <Field label="External player" hint="The engine hands the local stream URL to the player process.">
            <Select
              value={config?.player ?? 'auto'}
              onChange={event => void choosePlayer(event.target.value)}
            >
              <option value="auto">Auto (mpv, then VLC)</option>
              <option value="mpv">mpv</option>
              <option value="vlc">VLC</option>
              <option value="none">Never launch</option>
            </Select>
          </Field>
          <div className="mt-3 rounded-lg border border-[var(--border-soft)] bg-[var(--surface-2)] px-3 py-2 text-[11px] leading-relaxed text-[var(--muted)]">
            MP4/H.264 and WebM play inline. MKV/HEVC, DTS, and complex subtitles are better in a native
            player; use the player page's hand-off button.
          </div>
        </SettingsSection>
      </div>

      <SettingsSection
        icon={<Search className="size-4" />}
        title="Search API"
        description="Any JSON endpoint returning an array works. Use {query} and {page} placeholders in the URL."
        action={
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" loading={testingSearch} onClick={() => void testSearch()}>
              <RefreshCw className="size-3.5" />
              Test
            </Button>
            <Button variant="primary" size="sm" loading={savingSearch} onClick={() => void saveSearch()}>
              Save
            </Button>
          </div>
        }
      >
        {searchConfig.isLoading ? (
          <SkeletonRows count={2} />
        ) : (
          <div className="space-y-4">
            <Field
              label="Endpoint URL template"
              hint="Example: https://api.example.com/search?q={query}&page={page}"
            >
              <TextInput
                value={searchUrl}
                onChange={event => setSearchUrl(event.target.value)}
                placeholder="https://api.example.com/search?q={query}"
                icon={<ExternalLink className="size-4" />}
              />
            </Field>
            <Field label="Headers (JSON)" hint="Optional. API keys are stored in search.json with 0600 permissions.">
              <textarea
                value={headersText}
                onChange={event => setHeadersText(event.target.value)}
                rows={4}
                spellCheck={false}
                className="input h-auto min-h-[96px] resize-y py-2.5 font-mono text-xs leading-relaxed"
              />
            </Field>
            <div>
              <Label>Field map</Label>
              <div className="mt-2 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {(Object.keys(fieldMap) as Array<keyof FieldMap>).map(key => (
                  <Field key={key} label={key}>
                    <TextInput
                      value={fieldMap[key]}
                      onChange={event => setFieldMap(current => ({ ...current, [key]: event.target.value }))}
                      className="font-mono text-xs"
                    />
                  </Field>
                ))}
              </div>
            </div>
          </div>
        )}
      </SettingsSection>

      <div className="grid gap-4 lg:grid-cols-2">
        <SettingsSection
          icon={<Server className="size-4" />}
          title="Engine"
          description="The daemon is a local Rust process. No Node server runs at runtime."
        >
          <dl className="space-y-2 text-xs">
            <div className="flex items-center justify-between gap-3">
              <dt className="text-[var(--muted)]">Connection</dt>
              <dd className="font-mono text-[var(--success)]">{connected ? 'live' : 'offline'}</dd>
            </div>
            <div className="flex items-center justify-between gap-3">
              <dt className="text-[var(--muted)]">Version</dt>
              <dd className="font-mono text-[var(--fg)]">{bootstrap?.version ?? '—'}</dd>
            </div>
            <div className="flex items-start justify-between gap-3">
              <dt className="shrink-0 text-[var(--muted)]">Download dir</dt>
              <dd className="break-all text-right font-mono text-[var(--fg)]">{config?.downloadDir ?? '—'}</dd>
            </div>
          </dl>
        </SettingsSection>

        <SettingsSection
          icon={<Database className="size-4" />}
          title="Local data"
          description="History, config, and the torrent session never leave this machine unless you open a remote bind."
        >
          <div className="flex flex-wrap gap-2">
            <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1 text-[10px] font-medium text-[var(--muted)]">
              history.json
            </span>
            <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1 text-[10px] font-medium text-[var(--muted)]">
              search.json
            </span>
            <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1 text-[10px] font-medium text-[var(--muted)]">
              server-token
            </span>
            <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1 text-[10px] font-medium text-[var(--muted)]">
              librqbit session
            </span>
          </div>
          <p className="mt-3 flex items-center gap-1.5 text-[11px] text-[var(--muted)]">
            <Monitor className="size-3.5" />
            Single-user, loopback-first. Remote mode requires an explicit token.
          </p>
        </SettingsSection>
      </div>
    </div>
  )
}
