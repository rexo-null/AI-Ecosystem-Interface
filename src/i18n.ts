type Language = 'en' | 'ru';

const translations = {
  ru: {
    header: {
      title: 'ISKIN - AI IDE',
      runAgent: 'Запустить агента',
      sandbox: 'Песочница',
    },
    sidebar: {
      files: 'Файлы',
      knowledge: 'Знания',
    },
    editor: {
      placeholder: 'Откройте файл для просмотра',
      activeFile: 'Активный файл:',
    },
    chat: {
      title: 'AI Ассистент',
      placeholder: 'Спросите ISKIN...',
      send: 'Отправить',
      loading: 'Печатает...',
    },
    knowledge: {
      all: 'Все',
      constitution: 'Конституция',
      protocol: 'Протокол',
      pattern: 'Паттерн',
      tool: 'Инструмент',
      title: 'Заголовок',
      type: 'Тип',
      priority: 'Приоритет',
      tags: 'Теги',
      addNew: 'Добавить запись',
      edit: 'Редактировать',
      delete: 'Удалить',
    },
    terminal: {
      title: 'Терминал',
      clear: 'Очистить',
    },
    fileTree: {
      loading: 'Загрузка файлов...',
      error: 'Ошибка загрузки',
    },
  },
  en: {
    header: {
      title: 'ISKIN - AI IDE',
      runAgent: 'Run Agent',
      sandbox: 'Sandbox',
    },
    sidebar: {
      files: 'Files',
      knowledge: 'Knowledge',
    },
    editor: {
      placeholder: 'Open a file to view',
      activeFile: 'Active file:',
    },
    chat: {
      title: 'AI Assistant',
      placeholder: 'Ask ISKIN anything...',
      send: 'Send',
      loading: 'Typing...',
    },
    knowledge: {
      all: 'All',
      constitution: 'Constitution',
      protocol: 'Protocol',
      pattern: 'Pattern',
      tool: 'Tool',
      title: 'Title',
      type: 'Type',
      priority: 'Priority',
      tags: 'Tags',
      addNew: 'Add Entry',
      edit: 'Edit',
      delete: 'Delete',
    },
    terminal: {
      title: 'Terminal',
      clear: 'Clear',
    },
    fileTree: {
      loading: 'Loading files...',
      error: 'Error loading files',
    },
  },
};

let currentLanguage: Language = 'ru';

export const i18n = {
  setLanguage: (lang: Language) => {
    currentLanguage = lang;
    localStorage.setItem('language', lang);
  },
  
  getLanguage: () => currentLanguage,
  
  initLanguage: () => {
    const saved = localStorage.getItem('language') as Language;
    if (saved && (saved === 'ru' || saved === 'en')) {
      currentLanguage = saved;
    }
  },
  
  t: (path: string): string => {
    const keys = path.split('.');
    let value: any = translations[currentLanguage];
    
    for (const key of keys) {
      value = value?.[key];
    }
    
    return value || path;
  },
};

i18n.initLanguage();
