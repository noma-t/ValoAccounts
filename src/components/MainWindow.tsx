interface MainWindowProps {
  children?: React.ReactNode
}

export function MainWindow({ children }: MainWindowProps) {
  return (
    <main
      data-testid="main-window"
      className="flex-1 bg-neutral-800 text-white overflow-auto"
    >
      <div className="px-3 h-full">
        {children}
      </div>
    </main>
  )
}
