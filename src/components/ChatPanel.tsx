import { useState } from 'react';
import { i18n } from '../i18n';
import { useEditorStore } from '../store';
import { invoke } from '@tauri-apps/api/core';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  isLoading?: boolean;
}

export default function ChatPanel() {
  const [messages, setMessages] = useState<Message[]>([
    { 
      id: '1', 
      role: 'assistant', 
      content: 'Привет! Я ISKIN, ваш AI ассистент. Как я могу вам помочь?', 
      timestamp: Date.now() 
    }
  ]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const activeFile = useEditorStore(state => state.activeFile);

  const sendMessage = async () => {
    if (!input.trim()) return;
    
    const userMessage = input;
    const newMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: userMessage,
      timestamp: Date.now()
    };
    
    setMessages(prev => [...prev, newMessage]);
    setInput('');
    setIsLoading(true);
    
    try {
      // Вызиваем LLM команду из Tauri backend
      const response = await invoke<{ content: string; model: string; tokens_used: number }>(
        'chat_with_llm',
        {
          user_message: userMessage,
          system_prompt: activeFile 
            ? `Ты анализируешь файл: ${activeFile.path}\n\nСодержимое:\n${activeFile.content.substring(0, 2000)}`
            : 'Ты помощник в IDE для программирования. Помогай пользователю с кодом и ответами на вопросы.',
        }
      );

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: Date.now(),
      };
      
      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Error communicating with LLM:', error);
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `❌ Ошибка при подключении к LLM: ${error}`,
        timestamp: Date.now(),
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '12px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <strong>{i18n.t('chat.title')}</strong>
        <button
          onClick={() => i18n.setLanguage(i18n.getLanguage() === 'ru' ? 'en' : 'ru')}
          style={{
            background: 'var(--bg-tertiary)',
            border: 'none',
            borderRadius: '4px',
            color: 'var(--text-primary)',
            padding: '4px 8px',
            cursor: 'pointer',
            fontSize: '10px',
          }}
        >
          {i18n.getLanguage() === 'ru' ? 'EN' : 'RU'}
        </button>
      </div>
      
      <div className="chat-messages" style={{
        flex: 1,
        overflow: 'auto',
        padding: '12px',
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
      }}>
        {messages.map(msg => (
          <div 
            key={msg.id} 
            className={`chat-message ${msg.role}`}
            style={{
              display: 'flex',
              justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
              gap: '8px',
            }}
          >
            <div 
              style={{ 
                maxWidth: '80%',
                padding: '10px 12px',
                borderRadius: '8px',
                background: msg.role === 'user' ? 'var(--accent)' : 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                fontSize: '13px', 
                lineHeight: '1.4',
                wordWrap: 'break-word',
                whiteSpace: 'pre-wrap',
              }}
            >
              {msg.content}
            </div>
          </div>
        ))}
        {isLoading && (
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', color: 'var(--text-secondary)' }}>
            <div style={{
              display: 'flex',
              gap: '4px',
              padding: '10px 12px',
            }}>
              <span style={{ animation: 'pulse 1s infinite' }}>●</span>
              <span style={{ animation: 'pulse 1s infinite 0.2s' }}>●</span>
              <span style={{ animation: 'pulse 1s infinite 0.4s' }}>●</span>
            </div>
          </div>
        )}
      </div>
      
      <div className="chat-input" style={{
        padding: '12px',
        borderTop: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
      }}>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
          placeholder={i18n.t('chat.placeholder')}
          disabled={isLoading}
          style={{
            padding: '8px',
            borderRadius: '4px',
            border: '1px solid var(--border)',
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            fontSize: '12px',
            minHeight: '40px',
            resize: 'vertical',
            fontFamily: 'inherit',
          }}
        />
        <button 
          onClick={sendMessage}
          disabled={isLoading || !input.trim()}
          style={{
            padding: '8px 12px',
            background: isLoading ? 'var(--border)' : 'var(--accent)',
            border: 'none',
            borderRadius: '4px',
            color: 'white',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            fontSize: '12px',
            fontWeight: 'bold',
            transition: 'background-color 0.2s',
          }}
        >
          {isLoading ? i18n.t('chat.loading') : i18n.t('chat.send')}
        </button>
        {activeFile && (
          <div style={{
            fontSize: '10px',
            color: 'var(--text-secondary)',
            padding: '4px 0',
            borderTop: '1px solid var(--border)',
            paddingTop: '8px',
          }}>
            📄 {activeFile.path.split('/').pop()}
          </div>
        )}
      </div>
      
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 0.3; }
          50% { opacity: 1; }
        }
      `}</style>
    </div>
  );
}