import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

type ToastType = 'success' | 'error' | 'warning'

interface ToastItem {
  id: number
  type: ToastType
  message: string
  exiting: boolean
}

interface ToastContextValue {
  toast: (type: ToastType, message: string) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

const TOAST_DURATION = 3000
const ENTER_DURATION = 200
const EXIT_DURATION = 200

const TYPE_BORDER: Record<ToastType, string> = {
  success: 'border-l-green-400',
  error: 'border-l-red-400',
  warning: 'border-l-yellow-400',
}

const TYPE_PROGRESS: Record<ToastType, string> = {
  success: 'bg-green-400',
  error: 'bg-red-400',
  warning: 'bg-yellow-400',
}

function ToastItemComponent({ item }: { item: ToastItem }) {
  const [entered, setEntered] = useState(false)
  const [progress, setProgress] = useState(100)

  useEffect(() => {
    let raf1: number
    let raf2: number
    raf1 = requestAnimationFrame(() => {
      raf2 = requestAnimationFrame(() => {
        setEntered(true)
        setProgress(0)
      })
    })
    return () => {
      cancelAnimationFrame(raf1)
      cancelAnimationFrame(raf2)
    }
  }, [])

  const isVisible = entered && !item.exiting
  const animDuration = item.exiting ? EXIT_DURATION : ENTER_DURATION

  return (
    <div
      style={{
        opacity: isVisible ? 1 : 0,
        transform: isVisible ? 'translateX(0)' : 'translateX(12px)',
        transition: `opacity ${animDuration}ms ease, transform ${animDuration}ms ease`,
      }}
      className={`relative overflow-hidden bg-neutral-800 border border-neutral-700/50 border-l-2 ${TYPE_BORDER[item.type]} rounded px-3 py-2 text-sm text-white shadow-lg min-w-[160px] max-w-[260px]`}
    >
      {item.message}
      <div
        style={{ width: `${progress}%`, transition: `width ${TOAST_DURATION}ms linear` }}
        className={`absolute bottom-0 left-0 h-0.5 ${TYPE_PROGRESS[item.type]}`}
      />
    </div>
  )
}

function ToastContainer({ toasts }: { toasts: ToastItem[] }) {
  if (toasts.length === 0) return null

  return (
    <div className="fixed bottom-3 right-3 flex flex-col gap-2 z-50 pointer-events-none">
      {toasts.map((t) => (
        <ToastItemComponent key={t.id} item={t} />
      ))}
    </div>
  )
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([])

  const toast = useCallback((type: ToastType, message: string) => {
    const id = Date.now()
    setToasts((prev) => [...prev, { id, type, message, exiting: false }])

    setTimeout(() => {
      setToasts((prev) => prev.map((t) => (t.id === id ? { ...t, exiting: true } : t)))
    }, TOAST_DURATION - EXIT_DURATION)

    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id))
    }, TOAST_DURATION)
  }, [])

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <ToastContainer toasts={toasts} />
    </ToastContext.Provider>
  )
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext)
  if (!ctx) {
    throw new Error('useToast must be used within ToastProvider')
  }
  return ctx
}
