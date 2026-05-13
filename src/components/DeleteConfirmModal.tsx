import { MODAL_STYLES } from './shared'

interface DeleteConfirmModalProps {
  accountName: string
  onCancel: () => void
  onConfirm: () => void
}

export function DeleteConfirmModal({ accountName, onCancel, onConfirm }: DeleteConfirmModalProps) {
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-[60]">
      <div className="bg-neutral-900 border border-neutral-700/70 rounded-lg p-5 w-full max-w-xs mx-4 shadow-2xl">
        <p className="text-sm font-semibold text-white mb-1">Delete account?</p>
        <p className="text-xs text-neutral-200 mb-4">{accountName}</p>
        <div className="flex justify-end gap-2">
          <button
            type="button"
            className={MODAL_STYLES.cancelButton}
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className={MODAL_STYLES.submitButton}
            onClick={onConfirm}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  )
}
