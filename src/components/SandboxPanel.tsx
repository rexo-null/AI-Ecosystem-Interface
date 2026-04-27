import { useState, useEffect } from 'react';
import { useEditorStore } from '../store';

const STATUS_COLORS: Record<string, string> = {
  running: '#4caf50',
  created: '#2196f3',
  stopped: '#9e9e9e',
  exited: '#ff9800',
  error: '#f44336',
  paused: '#ff9800',
  unknown: '#9e9e9e',
};

const STATUS_ICONS: Record<string, string> = {
  running: '●',
  created: '○',
  stopped: '■',
  exited: '✕',
  error: '!',
  paused: '❚❚',
  unknown: '?',
};

export default function SandboxPanel() {
  const {
    sandboxStatus,
    containers,
    sandboxLoading,
    loadSandboxStatus,
    loadContainers,
    createContainer,
    startContainer,
    stopContainer,
    removeContainer,
    launchBrowser,
    closeBrowser,
  } = useEditorStore();

  const [activeTab, setActiveTab] = useState<'containers' | 'browser' | 'health'>('containers');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newImage, setNewImage] = useState('ubuntu:22.04');
  const [newName, setNewName] = useState('');
  const [newMemory, setNewMemory] = useState('512');

  useEffect(() => {
    loadSandboxStatus();
    loadContainers();
  }, []);

  const handleCreate = async () => {
    if (!newImage.trim()) return;
    await createContainer({
      image: newImage,
      name: newName || undefined,
      memory_limit_mb: parseInt(newMemory) || 512,
    });
    setShowCreateForm(false);
    setNewImage('ubuntu:22.04');
    setNewName('');
    setNewMemory('512');
  };

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', color: '#cccccc' }}>
      {/* Header */}
      <div style={{ padding: '8px 12px', borderBottom: '1px solid #333', display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{ fontWeight: 600, fontSize: 13 }}>Sandbox</span>
        {sandboxLoading && <span style={{ fontSize: 11, color: '#888' }}>загрузка...</span>}
        <button
          onClick={() => { loadSandboxStatus(); loadContainers(); }}
          style={{
            marginLeft: 'auto', background: 'none', border: '1px solid #555',
            color: '#ccc', padding: '2px 8px', cursor: 'pointer', borderRadius: 3, fontSize: 11,
          }}
        >
          ↻
        </button>
      </div>

      {/* Docker Status */}
      {sandboxStatus && (
        <div style={{ padding: '6px 12px', borderBottom: '1px solid #333', fontSize: 11, display: 'flex', gap: 12, flexWrap: 'wrap' }}>
          <span>
            Docker:{' '}
            <span style={{ color: sandboxStatus.docker.connected ? '#4caf50' : '#f44336' }}>
              {sandboxStatus.docker.connected ? 'подключён' : 'не доступен'}
            </span>
          </span>
          {sandboxStatus.docker.version && (
            <span style={{ color: '#888' }}>v{sandboxStatus.docker.version}</span>
          )}
          <span>Контейнеры: {sandboxStatus.containers_count}</span>
          <span>VNC: {sandboxStatus.vnc_sessions_count}</span>
          <span>
            Браузер:{' '}
            <span style={{ color: sandboxStatus.browser_status === 'running' ? '#4caf50' : '#888' }}>
              {sandboxStatus.browser_status}
            </span>
          </span>
        </div>
      )}

      {/* Tabs */}
      <div style={{ display: 'flex', borderBottom: '1px solid #333' }}>
        {(['containers', 'browser', 'health'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              flex: 1, padding: '6px 8px', background: activeTab === tab ? '#2d2d2d' : 'transparent',
              border: 'none', borderBottom: activeTab === tab ? '2px solid #007acc' : '2px solid transparent',
              color: activeTab === tab ? '#fff' : '#888', cursor: 'pointer', fontSize: 12,
            }}
          >
            {tab === 'containers' ? 'Контейнеры' : tab === 'browser' ? 'Браузер' : 'Здоровье'}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '8px 12px' }}>
        {activeTab === 'containers' && (
          <div>
            <button
              onClick={() => setShowCreateForm(!showCreateForm)}
              style={{
                width: '100%', padding: '6px', marginBottom: 8, background: '#0e639c',
                border: 'none', color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 12,
              }}
            >
              + Создать контейнер
            </button>

            {showCreateForm && (
              <div style={{ padding: 8, marginBottom: 8, border: '1px solid #444', borderRadius: 4, background: '#1e1e2e' }}>
                <div style={{ marginBottom: 6 }}>
                  <label style={{ fontSize: 11, color: '#888', display: 'block', marginBottom: 2 }}>Образ (image)</label>
                  <input
                    value={newImage}
                    onChange={(e) => setNewImage(e.target.value)}
                    placeholder="ubuntu:22.04"
                    style={{
                      width: '100%', padding: '4px 6px', background: '#2d2d2d', border: '1px solid #555',
                      color: '#ccc', borderRadius: 3, fontSize: 12, boxSizing: 'border-box',
                    }}
                  />
                </div>
                <div style={{ marginBottom: 6 }}>
                  <label style={{ fontSize: 11, color: '#888', display: 'block', marginBottom: 2 }}>Имя (опционально)</label>
                  <input
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    placeholder="my-sandbox"
                    style={{
                      width: '100%', padding: '4px 6px', background: '#2d2d2d', border: '1px solid #555',
                      color: '#ccc', borderRadius: 3, fontSize: 12, boxSizing: 'border-box',
                    }}
                  />
                </div>
                <div style={{ marginBottom: 6 }}>
                  <label style={{ fontSize: 11, color: '#888', display: 'block', marginBottom: 2 }}>Память (MB)</label>
                  <input
                    type="number"
                    value={newMemory}
                    onChange={(e) => setNewMemory(e.target.value)}
                    style={{
                      width: '100%', padding: '4px 6px', background: '#2d2d2d', border: '1px solid #555',
                      color: '#ccc', borderRadius: 3, fontSize: 12, boxSizing: 'border-box',
                    }}
                  />
                </div>
                <div style={{ display: 'flex', gap: 6 }}>
                  <button
                    onClick={handleCreate}
                    style={{ flex: 1, padding: '4px', background: '#0e639c', border: 'none', color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 12 }}
                  >
                    Создать
                  </button>
                  <button
                    onClick={() => setShowCreateForm(false)}
                    style={{ flex: 1, padding: '4px', background: '#333', border: 'none', color: '#ccc', cursor: 'pointer', borderRadius: 3, fontSize: 12 }}
                  >
                    Отмена
                  </button>
                </div>
              </div>
            )}

            {containers.length === 0 ? (
              <div style={{ fontSize: 12, color: '#666', textAlign: 'center', padding: 20 }}>
                Нет контейнеров
              </div>
            ) : (
              containers.map((c) => (
                <div
                  key={c.id}
                  style={{
                    padding: 8, marginBottom: 6, border: '1px solid #333',
                    borderRadius: 4, background: '#1e1e2e',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
                    <span style={{ color: STATUS_COLORS[c.status] || '#888', fontSize: 14 }}>
                      {STATUS_ICONS[c.status] || '?'}
                    </span>
                    <span style={{ fontSize: 12, fontWeight: 600, flex: 1 }}>
                      {c.name || c.id}
                    </span>
                    <span style={{ fontSize: 10, color: '#888' }}>{c.status}</span>
                  </div>
                  <div style={{ fontSize: 11, color: '#888', marginBottom: 4 }}>{c.image}</div>
                  <div style={{ display: 'flex', gap: 4 }}>
                    {c.status !== 'running' && (
                      <button
                        onClick={() => startContainer(c.id)}
                        style={{ padding: '2px 8px', background: '#2ea04370', border: '1px solid #2ea043', color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 10 }}
                      >
                        Запустить
                      </button>
                    )}
                    {c.status === 'running' && (
                      <button
                        onClick={() => stopContainer(c.id)}
                        style={{ padding: '2px 8px', background: '#d2942070', border: '1px solid #d29420', color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 10 }}
                      >
                        Остановить
                      </button>
                    )}
                    <button
                      onClick={() => removeContainer(c.id)}
                      style={{ padding: '2px 8px', background: '#da363370', border: '1px solid #da3633', color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 10 }}
                    >
                      Удалить
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'browser' && (
          <div>
            <div style={{ marginBottom: 12 }}>
              <div style={{ fontSize: 12, marginBottom: 8, color: '#ccc' }}>
                Headless Chrome (CDP) — автоматизация браузера для тестирования UI
              </div>
              <div style={{ display: 'flex', gap: 6 }}>
                <button
                  onClick={launchBrowser}
                  style={{
                    padding: '6px 12px', background: '#0e639c', border: 'none',
                    color: '#fff', cursor: 'pointer', borderRadius: 3, fontSize: 12,
                  }}
                >
                  Запустить Chrome
                </button>
                <button
                  onClick={closeBrowser}
                  style={{
                    padding: '6px 12px', background: '#333', border: '1px solid #555',
                    color: '#ccc', cursor: 'pointer', borderRadius: 3, fontSize: 12,
                  }}
                >
                  Закрыть
                </button>
              </div>
            </div>
            {sandboxStatus && (
              <div style={{ fontSize: 11, color: '#888' }}>
                <div>Статус: <span style={{ color: sandboxStatus.browser_status === 'running' ? '#4caf50' : '#888' }}>{sandboxStatus.browser_status}</span></div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'health' && (
          <div>
            <div style={{ fontSize: 12, marginBottom: 8, color: '#ccc' }}>
              Self-Healing — автоматический мониторинг и восстановление контейнеров
            </div>
            {sandboxStatus && (
              <div style={{ fontSize: 11, display: 'flex', flexDirection: 'column', gap: 4 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Всего проверок:</span>
                  <span>{sandboxStatus.healing_stats.total_checks}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Восстановлений:</span>
                  <span>{sandboxStatus.healing_stats.total_recoveries}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Успешных:</span>
                  <span style={{ color: '#4caf50' }}>{sandboxStatus.healing_stats.successful_recoveries}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Неудачных:</span>
                  <span style={{ color: sandboxStatus.healing_stats.failed_recoveries > 0 ? '#f44336' : '#888' }}>
                    {sandboxStatus.healing_stats.failed_recoveries}
                  </span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Мониторов:</span>
                  <span>{sandboxStatus.healing_stats.active_monitors}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #333' }}>
                  <span style={{ color: '#888' }}>Паттернов ошибок:</span>
                  <span>{sandboxStatus.healing_stats.error_patterns_count}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0' }}>
                  <span style={{ color: '#888' }}>Uptime:</span>
                  <span style={{ color: '#4caf50' }}>{sandboxStatus.healing_stats.uptime_percentage.toFixed(1)}%</span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
