import { useState, useEffect, useRef } from 'react'

export function useCountdown(initialSecs: number | null): number | null {
  const [remaining, setRemaining] = useState(initialSecs)
  const startRef = useRef<number | null>(null)
  const initialRef = useRef(initialSecs)

  useEffect(() => {
    initialRef.current = initialSecs
    if (initialSecs === null || initialSecs <= 0) {
      setRemaining(initialSecs)
      startRef.current = null
      return
    }

    startRef.current = Date.now()
    setRemaining(initialSecs)

    const id = setInterval(() => {
      const elapsed = Math.floor((Date.now() - (startRef.current ?? Date.now())) / 1000)
      const next = (initialRef.current ?? 0) - elapsed
      setRemaining(next > 0 ? next : 0)
    }, 1000)

    return () => clearInterval(id)
  }, [initialSecs])

  return remaining
}
