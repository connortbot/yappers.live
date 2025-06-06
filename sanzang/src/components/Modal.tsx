import { useEffect } from 'react'

interface ModalProps {
  isOpen: boolean
  onClose: () => void
  children: React.ReactNode
  title?: string
}

export function Modal({ isOpen, onClose, children, title }: ModalProps) {
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    if (isOpen) {
      document.addEventListener('keydown', handleEscape)
    }

    return () => {
      document.removeEventListener('keydown', handleEscape)
    }
  }, [isOpen, onClose])

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
      {/* Modal Content */}
      <div className="relative bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto transform transition-all duration-300 scale-100 pointer-events-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-pencil border-opacity-20">
          {title && (
            <h2 className="text-2xl xs:text-3xl sm:text-4xl font-bold font-primary text-pencil">
              {title}
            </h2>
          )}
          <button
            onClick={onClose}
            className="text-pencil hover:text-primary transition-colors duration-200 text-2xl font-bold ml-auto"
          >
            ×
          </button>
        </div>
        
        {/* Body */}
        <div className="p-6">
          {children}
        </div>
      </div>
    </div>
  )
} 