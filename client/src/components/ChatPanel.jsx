import { useState, useRef, useEffect } from 'react'
import { sendMessage } from '../api/chat'

function ChatPanel() {
  const [messages, setMessages] = useState([
    {
      role: 'system',
      content: 'AI Ecosystem Interface готов к работе. Я использую Qwen 2.5 14b Coder через llama.cpp для помощи в разработке.'
    }
  ])
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const messagesEndRef = useRef(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!inputValue.trim() || isLoading) return

    const userMessage = { role: 'user', content: inputValue.trim() }
    setMessages(prev => [...prev, userMessage])
    setInputValue('')
    setIsLoading(true)

    try {
      const response = await sendMessage(userMessage.content)
      setMessages(prev => [...prev, { role: 'assistant', content: response }])
    } catch (error) {
      setMessages(prev => [...prev, { 
        role: 'system', 
        content: `Ошибка: ${error.message}` 
      }])
    } finally {
      setIsLoading(false)
    }
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSubmit(e)
    }
  }

  return (
    <aside className="chat-panel">
      <div className="chat-header">
        AI Assistant (Qwen 2.5 14b)
      </div>
      
      <div className="chat-messages">
        {messages.map((message, index) => (
          <div 
            key={index} 
            className={`message ${message.role}`}
          >
            {message.content}
          </div>
        ))}
        {isLoading && (
          <div className="message assistant">
            <em>Печатает...</em>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input-container">
        <form onSubmit={handleSubmit}>
          <textarea
            className="chat-input"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Спросите что-нибудь или попросите изменить код..."
            rows={3}
            disabled={isLoading}
          />
          <button 
            type="submit" 
            className="send-button"
            disabled={isLoading || !inputValue.trim()}
          >
            Отправить
          </button>
        </form>
      </div>
    </aside>
  )
}

export default ChatPanel
