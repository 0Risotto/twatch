import { useEffect, useRef } from 'react'
import type { ChangeEvent as ReactChangeEvent } from 'react'
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom'
import { usePlayback } from '@/features/player/PlaybackProvider'
import { canPreview, fileKind, isAudio, isImage, isVideo } from '@/lib/fileIcons'
import { useAppStore } from '@/store/app'
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
    download: activeDownload,
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
    registerSlot,
  } = usePlayback()

  const requestedId = Number(params.get('download')) || null
  const autoDownload =
    !requestedId && !activeDownload
      ? (downloads.find(item => item.streamPath) ??
        downloads.find(item => item.isStreaming) ??
        null)
      : null
  const autoId = autoDownload?.id ?? null

  useEffect(() => {
    if (requestedId) activate(requestedId)
  }, [activate, requestedId])

  useEffect(() => {
    if (autoId) activate(autoId)
  }, [activate, autoId])

  const download = requestedId
    ? (downloads.find(item => item.id === requestedId) ?? activeDownload)
    : (activeDownload ?? autoDownload)

  const streamUrl = download?.streamPath ?? null
  const kind = fileKind(download?.fileNames[0] ?? download?.label ?? '')
  const video = isVideo(kind)
  const audio = isAudio(kind)
  const image = isImage(kind)
  const previewable = canPreview(kind)
  const playable = video || audio
  const queue = download?.queue ?? []
  const queueIndex = queue.findIndex(item => item.index === (download?.fileIndex ?? 0))
  const preparing = Boolean(download) && !streamUrl && download?.state !== 'failed'
  const waitingForPeers = Boolean(download) && download?.peers === 0 && download?.speed === 0
  const bufferedAhead = Math.max(
    0,
    ...player.state.buffered.map(range => range.end - player.state.currentTime),
  )
  const error = Boolean(download) && (player.state.error || download?.state === 'failed')
  const dockActive = Boolean(download && streamUrl && playable && !error)

  const goBack = () => {
    const from = (location.state as { from?: string } | null)?.from
    if (from) {
      navigate(from)
      return
    }
    if (window.history.length > 1) navigate(-1)
    else navigate('/downloads')
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

  if (!download) return <PlayerEmptyState />

  return (
    <div className="space-y-4">
      <PlayerHeader
        download={download}
        streamUrl={streamUrl}
        previewable={previewable}
        onBack={goBack}
      />

      <div className="grid gap-4 xl:grid-cols-[1fr_320px]">
        <PlayerSlot
          download={download}
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
          download={download}
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
