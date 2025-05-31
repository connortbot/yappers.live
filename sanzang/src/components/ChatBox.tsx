import { useState } from 'react'
import { Input } from './Input'
import { Button } from './Button'
import { FormRow } from './FormRow'
import type { GameMessage } from '../lib/bindings/GameMessage'

interface ChatBoxProps {
  messages: string[]
  onSendMessage: (message: GameMessage) => void
  placeholder?: string
  username: string
}

export function ChatBox({ messages, onSendMessage, placeholder = "Type a message", username }: ChatBoxProps) {
  const [message, setMessage] = useState('')

  const handleSend = () => {
    if (message.trim()) {
      const chatMessage: GameMessage = {
        type: 'ChatMessage',
        username: username,
        message: message.trim(),
      }
      onSendMessage(chatMessage)
      setMessage('')
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSend()
    }
  }

  return (
    <>
      <div className="mb-4 h-32 sm:h-48 overflow-y-auto border-2 border-pencil rounded p-2">
        {messages.map((msg, index) => (
          <div key={index} className="mb-1 font-secondary text-pencil text-sm sm:text-base break-words">
            {msg}
          </div>
        ))}
      </div>
      <FormRow>
        <Input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder={placeholder}
          className="flex-1"
          onKeyDown={handleKeyDown}
        />
        <Button
          variant="secondary"
          size="medium"
          onMouseUp={handleSend}
          className="w-full sm:w-auto"
        >
          Send
        </Button>
      </FormRow>
    </>
  )
} 