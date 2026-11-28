import { useEffect, useRef } from 'react'
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom'

import { useQuery } from '@tanstack/react-query'
import type { ChangeEvent as ReactChangeEvent } from 'react'

import { api } from '@/lib/api'
import { fileKind } from '@/lib/fileIcons'

import { useAppStore } from '@/store/app'

import { usePlayback } from '@/features/player/playbackContext'
import { downloadTrack } from '@/features/player/playbackTrack'

import { PlayerEmptyState } from './player/PlayerEmptyState'
import { PlayerHeader } from './player/PlayerHeader'
import { PlayerSidebar } from './player/PlayerSidebar'
import { PlayerSlot } from './player/PlayerSlot'

export function PlayerPage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const location = useLocation()
  const downloads = useAppStore(state => state.downloads)
  const addToast = useAppStore(state => state.addToast)
  const subtitleInputRef = useRef<HTMLInputElement | null>(null)
  const {
    track: activeTrack,
    cover,
    player,
    subtitles,
    externalSubtitle,
    setExternalSubtitle,
    switchTo,
    playPrev,
    playNext,
    hasPrev,
    hasNext,
    activate,
    activateLibrary,
    registerSlot,
  } = usePlayback()

  const requestedId = Number(params.get('download')) || null
  const requestedLibraryId = params.get('library')

  const library = useQuery({
    queryKey: ['library'],
    queryFn: api.library,
    enabled: Boolean(requestedLibraryId),
    staleTime: 30_000,
  })
  const libraryEntry =
    (requestedLibraryId && library.data?.find(entry => entry.id === requestedLibraryId)) || null

  const autoDownload =
    !requestedId && !requestedLibraryId && !activeTrack
      ? (downloads.find(item => item.media.streamPath) ??
        downloads.find(item => item.isStreaming) ??
        null)
      : null
  const autoId = autoDownload?.id ?? null

  useEffect(() => {
    if (requestedId) activate(requestedId)
  }, [activate, requestedId])

  useEffect(() => {
    if (libraryEntry) activateLibrary(libraryEntry)
  }, [activateLibrary, libraryEntry])

  useEffect(() => {
    if (autoId) activate(autoId)
  }, [activate, autoId])

  const requestedDownload = requestedId ? downloads.find(item => item.id === requestedId) : null
  const track = requestedLibraryId
    ? activeTrack?.source === 'library' && activeTrack.library?.id === requestedLibraryId
      ? activeTrack
      : null
    : requestedDownload
      ? downloadTrack(requestedDownload)
      : (activeTrack ?? (autoDownload ? downloadTrack(autoDownload) : null))

  const download = track?.download ?? null
  const streamUrl = track?.streamPath ?? null
  const kind = fileKind(track?.media.filename ?? '')
  const video = track?.media.kind === 'video'
  const audio = track?.media.kind === 'audio'
  const image = track?.media.kind === 'image'
  const previewable = video || audio || image
  const playable = video || audio
  const queue = track?.queue ?? []
  const queueIndex = queue.findIndex(item => item.index === (track?.fileIndex ?? 0))
  const preparing = Boolean(download) && !streamUrl && download?.state !== 'failed'
  const waitingForPeers = Boolean(download) && download?.peers === 0 && download?.speed === 0
  const bufferedAhead = Math.max(
    0,
    ...player.state.buffered.map(range => range.end - player.state.currentTime),
  )
  const error = Boolean(track) && (player.state.error || download?.state === 'failed')
  const dockActive = Boolean(track && streamUrl && playable && !error)

  const goBack = () => {
    const from = (location.state as { from?: string } | null)?.from
    if (from) {
      navigate(from)
      return
    }
    if (window.history.length > 1) navigate(-1)
    else navigate(track?.source === 'library' ? '/library' : '/downloads')
  }

  const copyStream = async () => {
    if (!streamUrl) return
    try {
      await navigator.clipboard.writeText(`${window.location.origin}${streamUrl}`)
      addToast('success', 'Stream URL copied')
    } catch {
      addToast('warning', 'Clipboard is unavailable')
    }
  }

  const loadSubtitle = async (event: ReactChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    if (!/\.(srt|vtt|ass|ssa)$/i.test(file.name)) {
      addToast('warning', 'Choose an SRT, VTT, ASS or SSA subtitle file')
      return
    }
    try {
      const content = await file.text()
      setExternalSubtitle({ name: file.name, content })
      addToast('success', `Loaded ${file.name}`)
    } catch {
      addToast('error', 'Could not read that subtitle file')
    }
  }

  if (!track) return <PlayerEmptyState />

  return (
    <div className="space-y-4">
      <PlayerHeader track={track} streamUrl={streamUrl} previewable={previewable} onBack={goBack} />

      <div className="grid gap-4 xl:grid-cols-[1fr_320px]">
        <PlayerSlot
          track={track}
          kind={kind}
          streamUrl={streamUrl}
          coverUrl={cover?.url ?? null}
          audio={audio}
          image={image}
          previewable={previewable}
          dockActive={dockActive}
          preparing={preparing}
          waitingForPeers={waitingForPeers}
          error={error}
          player={player}
          onRegisterSlot={registerSlot}
          onCopy={() => void copyStream()}
        />

        <PlayerSidebar
          track={track}
          kind={kind}
          video={video}
          audio={audio}
          image={image}
          playable={playable}
          streamUrl={streamUrl}
          bufferedAhead={bufferedAhead}
          queue={queue}
          queueIndex={queueIndex}
          hasPrev={hasPrev}
          hasNext={hasNext}
          onPrev={playPrev}
          onNext={playNext}
          onSwitch={switchTo}
          player={player}
          subtitles={subtitles}
          externalSubtitle={externalSubtitle}
          setExternalSubtitle={setExternalSubtitle}
          subtitleInputRef={subtitleInputRef}
          onLoadSubtitle={event => void loadSubtitle(event)}
        />
      </div>
    </div>
  )
}
