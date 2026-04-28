import { useState, useEffect, useRef, useCallback } from 'react';
import { i18n } from '../i18n';
import { useEditorStore } from '../store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  isStreaming?: boolean;
}

interface LLMStatusInfo {
  online: boolean;
  model: string;
  endpoint: string;
}

interface LLMTokenPayload {
  token: string;
  message_id: string;
}

interface LLMDonePayload {
  message_id: string;
  full_content: string;
  tokens_used: number;
}

interface LLMErrorPayload {
  message_id: string;
  error: string;
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
  const [llmStatus, setLlmStatus] = useState<LLMStatusInfo | null>(null);
  const [useStreaming, setUseStreaming] = useState(true);
  const activeFile = useEditorStore(state => state.activeFile);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const streamingContentRef = useRef<Record<string, string>>({});

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Check LLM status on mount
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const status = await invoke<LLMStatusInfo>('llm_status');
        setLlmStatus(status);
      } catch {
        setLlmStatus({ online: false, model: 'Unknown', endpoint: '' });
      }
    };
    checkStatus();
    const interval = setInterval(checkStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  // Listen for SSE streaming events
  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const unlisten1 = await listen<LLMTokenPayload>('llm-token', (event) => {
        const { token, message_id } = event.payload;
        streamingContentRef.current[message_id] =
          (streamingContentRef.current[message_id] || '') + token;

        setMessages(prev => prev.map(msg =>
          msg.id === message_id
            ? { ...msg, content: streamingContentRef.current[message_id] }
            : msg
        ));
      });
      unlisteners.push(unlisten1);

      const unlisten2 = await listen<LLMDonePayload>('llm-done', (event) => {
        const { message_id, full_content } = event.payload;
        delete streamingContentRef.current[message_id];

        setMessages(prev => prev.map(msg =>
          msg.id === message_id
            ? { ...msg, content: full_content, isStreaming: false }
            : msg
        ));
        setIsLoading(false);
      });
      unlisteners.push(unlisten2);

      const unlisten3 = await listen<LLMErrorPayload>('llm-error', (event) => {
        const { message_id, error } = event.payload;
        delete streamingContentRef.current[message_id];

        setMessages(prev => prev.map(msg =>
          msg.id === message_id
            ? { ...msg, content: `Ошибка: ${error}`, isStreaming: false }
            : msg
        ));
        setIsLoading(false);
      });
      unlisteners.push(unlisten3);
    };

    setup();
    return () => {
      unlisteners.forEach(fn => fn());
    };
  }, []);

  const stopGeneration = async () => {
    try {
      await invoke('llm_stop_generation');
    } catch (err) {
      console.error('Failed to stop generation:', err);
    }
  };

  const clearHistory = async () => {
    try {
      await invoke('llm_clear_history');
      setMessages([{
        id: Date.now().toString(),
        role: 'assistant',
        content: 'История очищена. Начнём новый разговор!',
        timestamp: Date.now()
      }]);
    } catch (err) {
      console.error('Failed to clear history:', err);
    }
  };

  const sendMessage = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage = input;
    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: userMessage,
      timestamp: Date.now()
    };

    const assistantId = (Date.now() + 1).toString();

    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setIsLoading(true);

    const systemPrompt = activeFile
      ? `Ты анализируешь файл: ${activeFile.path}\n\nСодержимое:\n${activeFile.content.substring(0, 2000)}`
      : 'Ты помощник в IDE для программирования. Помогай пользователю с кодом и ответами на вопросы.';

    if (useStreaming) {
      // Streaming mode: create placeholder message, backend sends events
      const streamMsg: Message = {
        id: assistantId,
        role: 'assistant',
        content: '',
        timestamp: Date.now(),
        isStreaming: true
      };
      setMessages(prev => [...prev, streamMsg]);
      streamingContentRef.current[assistantId] = '';

      try {
        await invoke('chat_with_llm_stream', {
          user_message: userMessage,
          system_prompt: systemPrompt,
          message_id: assistantId,
        });
      } catch (error) {
        setMessages(prev => prev.map(msg =>
          msg.id === assistantId
            ? { ...msg, content: `Ошибка: ${error}`, isStreaming: false }
            : msg
        ));
        setIsLoading(false);
      }
    } else {
      // Non-streaming mode: wait for full response
      try {
        const response = await invoke<{ content: string; model: string; tokens_used: number }>(
          'chat_with_llm',
          { user_message: userMessage, system_prompt: systemPrompt }
        );

        setMessages(prev => [...prev, {
          id: assistantId,
          role: 'assistant',
          content: response.content,
          timestamp: Date.now(),
        }]);
      } catch (error) {
        setMessages(prev => [...prev, {
          id: assistantId,
          role: 'assistant',
          content: `Ошибка при подключении к LLM: ${error}`,
          timestamp: Date.now(),
        }]);
      } finally {
        setIsLoading(false);
      }
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => {
      console.warn('Failed to copy to clipboard');
    });
  };

  const insertIntoEditor = (code: string) => {
    const store = useEditorStore.getState();
    if (store.activeFile) {
      store.setActiveFile({
        ...store.activeFile,
        content: store.activeFile.content + '\n' + code,
      });
    }
  };

  // Code block renderer with action buttons
  const CodeBlock = ({ children, className }: { children: string; className?: string }) => {
    const language = className?.replace('language-', '') || '';
    const code = String(children).replace(/\n$/, '');

    return (
      <div style={{ position: 'relative', marginBottom: '8px' }}>
        {language && (
          <div style={{
            background: 'var(--bg-primary)',
            color: 'var(--text-secondary)',
            padding: '2px 8px',
            fontSize: '10px',
            borderRadius: '4px 4px 0 0',
            borderBottom: '1px solid var(--border)',
          }}>
            {language}
          </div>
        )}
        <pre style={{
          background: 'var(--bg-primary)',
          padding: '10px',
          borderRadius: language ? '0 0 4px 4px' : '4px',
          overflow: 'auto',
          fontSize: '12px',
          lineHeight: '1.5',
          margin: 0,
        }}>
          <code>{code}</code>
        </pre>
        <div style={{
          display: 'flex',
          gap: '4px',
          position: 'absolute',
          top: language ? '24px' : '4px',
          right: '4px',
        }}>
          <button
            onClick={() => copyToClipboard(code)}
            title="Копировать код"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border)',
              borderRadius: '3px',
              color: 'var(--text-secondary)',
              padding: '2px 6px',
              cursor: 'pointer',
              fontSize: '10px',
            }}
          >
            Копировать
          </button>
          <button
            onClick={() => insertIntoEditor(code)}
            title="Вставить в редактор"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border)',
              borderRadius: '3px',
              color: 'var(--text-secondary)',
              padding: '2px 6px',
              cursor: 'pointer',
              fontSize: '10px',
            }}
          >
            В редактор
          </button>
          {(language === 'bash' || language === 'sh' || language === 'shell' || language === 'cmd' || language === 'powershell') && (
            <button
              onClick={() => {
                const termWrite = (window as unknown as Record<string, unknown>).__iskin_terminal_write;
                if (typeof termWrite === 'function') {
                  (termWrite as (cmd: string) => void)(code);
                }
              }}
              title="Выполнить в терминале"
              style={{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border)',
                borderRadius: '3px',
                color: 'var(--success)',
                padding: '2px 6px',
                cursor: 'pointer',
                fontSize: '10px',
              }}
            >
              Выполнить
            </button>
          )}
        </div>
      </div>
    );
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Header */}
      <div style={{
        padding: '8px 12px',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        gap: '8px',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <strong>{i18n.t('chat.title')}</strong>
          <div
            title={llmStatus?.online ? 'LLM Online' : 'LLM Offline'}
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              background: llmStatus?.online ? '#4caf50' : '#f44336',
            }}
          />
          {llmStatus && (
            <span style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>
              {llmStatus.online ? llmStatus.model : 'Offline'}
            </span>
          )}
        </div>
        <div style={{ display: 'flex', gap: '4px' }}>
          <button
            onClick={clearHistory}
            title="Очистить историю"
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
            Очистить
          </button>
          <button
            onClick={() => setUseStreaming(!useStreaming)}
            title={useStreaming ? 'Streaming включён' : 'Streaming выключен'}
            style={{
              background: useStreaming ? 'var(--accent)' : 'var(--bg-tertiary)',
              border: 'none',
              borderRadius: '4px',
              color: 'var(--text-primary)',
              padding: '4px 8px',
              cursor: 'pointer',
              fontSize: '10px',
            }}
          >
            Stream
          </button>
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
      </div>

      {/* Messages */}
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
                maxWidth: '85%',
                padding: '10px 12px',
                borderRadius: '8px',
                background: msg.role === 'user' ? 'var(--accent)' : 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                fontSize: '13px',
                lineHeight: '1.5',
                wordWrap: 'break-word',
                position: 'relative',
              }}
            >
              {msg.role === 'assistant' ? (
                <div className="markdown-content">
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                      code({ className, children, ...props }) {
                        const isInline = !className;
                        if (isInline) {
                          return (
                            <code
                              style={{
                                background: 'var(--bg-primary)',
                                padding: '1px 4px',
                                borderRadius: '3px',
                                fontSize: '12px',
                              }}
                              {...props}
                            >
                              {children}
                            </code>
                          );
                        }
                        return (
                          <CodeBlock className={className}>
                            {String(children)}
                          </CodeBlock>
                        );
                      },
                      pre({ children }) {
                        return <>{children}</>;
                      },
                      table({ children }) {
                        return (
                          <table style={{
                            borderCollapse: 'collapse',
                            width: '100%',
                            fontSize: '12px',
                            margin: '8px 0',
                          }}>
                            {children}
                          </table>
                        );
                      },
                      th({ children }) {
                        return (
                          <th style={{
                            border: '1px solid var(--border)',
                            padding: '4px 8px',
                            background: 'var(--bg-primary)',
                            textAlign: 'left',
                          }}>
                            {children}
                          </th>
                        );
                      },
                      td({ children }) {
                        return (
                          <td style={{
                            border: '1px solid var(--border)',
                            padding: '4px 8px',
                          }}>
                            {children}
                          </td>
                        );
                      },
                      a({ href, children }) {
                        return (
                          <a
                            href={href}
                            target="_blank"
                            rel="noopener noreferrer"
                            style={{ color: 'var(--accent)' }}
                          >
                            {children}
                          </a>
                        );
                      },
                    }}
                  >
                    {msg.content || ' '}
                  </ReactMarkdown>
                  {msg.isStreaming && (
                    <span style={{
                      display: 'inline-block',
                      width: '6px',
                      height: '14px',
                      background: 'var(--accent)',
                      animation: 'blink 1s infinite',
                      marginLeft: '2px',
                      verticalAlign: 'text-bottom',
                    }} />
                  )}
                </div>
              ) : (
                <span style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</span>
              )}
            </div>
          </div>
        ))}
        {isLoading && !messages.some(m => m.isStreaming) && (
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', color: 'var(--text-secondary)' }}>
            <div style={{
              display: 'flex',
              gap: '4px',
              padding: '10px 12px',
            }}>
              <span style={{ animation: 'pulse 1s infinite' }}>&#9679;</span>
              <span style={{ animation: 'pulse 1s infinite 0.2s' }}>&#9679;</span>
              <span style={{ animation: 'pulse 1s infinite 0.4s' }}>&#9679;</span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
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
        <div style={{ display: 'flex', gap: '8px' }}>
          {isLoading ? (
            <button
              onClick={stopGeneration}
              style={{
                flex: 1,
                padding: '8px 12px',
                background: '#f44336',
                border: 'none',
                borderRadius: '4px',
                color: 'white',
                cursor: 'pointer',
                fontSize: '12px',
                fontWeight: 'bold',
              }}
            >
              Остановить
            </button>
          ) : (
            <button
              onClick={sendMessage}
              disabled={!input.trim()}
              style={{
                flex: 1,
                padding: '8px 12px',
                background: !input.trim() ? 'var(--border)' : 'var(--accent)',
                border: 'none',
                borderRadius: '4px',
                color: 'white',
                cursor: !input.trim() ? 'not-allowed' : 'pointer',
                fontSize: '12px',
                fontWeight: 'bold',
                transition: 'background-color 0.2s',
              }}
            >
              {i18n.t('chat.send')}
            </button>
          )}
        </div>
        {activeFile && (
          <div style={{
            fontSize: '10px',
            color: 'var(--text-secondary)',
            padding: '4px 0',
            borderTop: '1px solid var(--border)',
            paddingTop: '8px',
          }}>
            {activeFile.path.split('/').pop()}
          </div>
        )}
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 0.3; }
          50% { opacity: 1; }
        }
        @keyframes blink {
          0%, 100% { opacity: 1; }
          50% { opacity: 0; }
        }
        .markdown-content p { margin: 4px 0; }
        .markdown-content ul, .markdown-content ol { margin: 4px 0; padding-left: 20px; }
        .markdown-content li { margin: 2px 0; }
        .markdown-content h1, .markdown-content h2, .markdown-content h3 {
          margin: 8px 0 4px 0;
          line-height: 1.3;
        }
        .markdown-content h1 { font-size: 16px; }
        .markdown-content h2 { font-size: 14px; }
        .markdown-content h3 { font-size: 13px; }
        .markdown-content blockquote {
          border-left: 3px solid var(--accent);
          padding-left: 8px;
          margin: 4px 0;
          color: var(--text-secondary);
        }
        .markdown-content hr {
          border: none;
          border-top: 1px solid var(--border);
          margin: 8px 0;
        }
      `}</style>
    </div>
  );
}
