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
      search: 'Поиск',
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
      userrule: 'Правило',
      tooldefinition: 'Инструмент',
      projectcontext: 'Контекст',
      title: 'Заголовок',
      type: 'Тип',
      priority: 'Приоритет',
      tags: 'Теги',
      addNew: 'Добавить запись',
      edit: 'Редактировать',
      delete: 'Удалить',
      search: 'Поиск...',
      noEntries: 'Нет записей',
      content: 'Содержимое',
      accessCount: 'Обращений',
    },
    codeSearch: {
      indexProject: 'Индексация проекта',
      projectPath: 'Путь к проекту',
      index: 'Индексировать',
      search: 'Поиск по коду',
      searchPlaceholder: 'Поиск символов и кода...',
      find: 'Найти',
      allLanguages: 'Все языки',
      noResults: 'Ничего не найдено',
      symbols: 'Символы',
      files: 'Файлов',
      lines: 'Строк',
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
      search: 'Search',
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
      userrule: 'Rule',
      tooldefinition: 'Tool',
      projectcontext: 'Context',
      title: 'Title',
      type: 'Type',
      priority: 'Priority',
      tags: 'Tags',
      addNew: 'Add Entry',
      edit: 'Edit',
      delete: 'Delete',
      search: 'Search...',
      noEntries: 'No entries',
      content: 'Content',
      accessCount: 'Access count',
    },
    codeSearch: {
      indexProject: 'Index Project',
      projectPath: 'Project path',
      index: 'Index',
      search: 'Code Search',
      searchPlaceholder: 'Search symbols and code...',
      find: 'Find',
      allLanguages: 'All languages',
      noResults: 'No results found',
      symbols: 'Symbols',
      files: 'Files',
      lines: 'Lines',
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
    let value: Record<string, unknown> | string = translations[currentLanguage] as Record<string, unknown>;
    
    for (const key of keys) {
      if (typeof value === 'object' && value !== null && key in value) {
        value = value[key] as Record<string, unknown> | string;
      } else {
        return path;
      }
    }
    
    return typeof value === 'string' ? value : path;
  },
};

i18n.initLanguage();
