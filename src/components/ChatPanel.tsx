import { useState } from 'react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

export default function ChatPanel() {
  const [messages, setMessages] = useState<Message[]>([
    { id: '1', role: 'assistant', content: 'Hello! I am ISKIN, your AI coding assistant. How can I help you today?', timestamp: Date.now() }
  ]);
  const [input, setInput] = useState('');

  const sendMessage = () => {
    if (!input.trim()) return;
    
    const newMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
      timestamp: Date.now()
    };
    
    setMessages([...messages, newMessage]);
    setInput('');
    
    // Simulate AI response (will be connected to LLM later)
    setTimeout(() => {
      const response: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'I understand your request. Let me analyze the code and provide assistance.',
        timestamp: Date.now()
      };
      setMessages(prev => [...prev, response]);
    }, 1000);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '12px', borderBottom: '1px solid var(--border)' }}>
        <strong>AI Assistant</strong>
      </div>
      
      <div className="chat-messages">
        {messages.map(msg => (
          <div 
            key={msg.id} 
            className={`chat-message ${msg.role}`}
          >
            <div style={{ fontSize: '13px', lineHeight: '1.5' }}>{msg.content}</div>
          </div>
        ))}
      </div>
      
      <div className="chat-input">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
          placeholder="Ask ISKIN anything..."
        />
        <button 
          className="btn" 
          style={{ marginTop: '8px', width: '100%' }}
          onClick={sendMessage}
        >
          Send
        </button>
      </div>
    </div>
  );
}