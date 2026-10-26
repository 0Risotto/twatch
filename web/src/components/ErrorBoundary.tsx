import { Component, type ErrorInfo, type ReactNode } from 'react'
import { AlertTriangle, Home, RotateCcw } from 'lucide-react'

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

/** Keeps a page-level runtime error from blanking the whole shell. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('twatch page error', error, info.componentStack)
  }

  render() {
    const { error } = this.state
    if (!error) return this.props.children

    return (
      <div className="panel mx-auto mt-10 max-w-xl p-6 text-center">
        <div className="mx-auto mb-4 grid size-12 place-items-center rounded-xl bg-[var(--danger-soft)] text-[var(--danger)]">
          <AlertTriangle className="size-6" />
        </div>
        <h1 className="text-lg font-semibold text-[var(--fg)]">This page hit an error</h1>
        <p className="mt-2 text-sm leading-relaxed text-[var(--muted)]">
          The engine is still running. Retry this page or go back home; your downloads and history
          are untouched.
        </p>
        <p className="mt-3 rounded-lg bg-[var(--surface-2)] px-3 py-2 font-mono text-[10px] text-[var(--danger)]">
          {error.message}
        </p>
        <div className="mt-5 flex items-center justify-center gap-2">
          <button
            type="button"
            className="btn btn-primary"
            onClick={() => this.setState({ error: null })}
          >
            <RotateCcw className="size-4" />
            Retry page
          </button>
          <a href="/" className="btn btn-secondary">
            <Home className="size-4" />
            Home
          </a>
        </div>
      </div>
    )
  }
}
