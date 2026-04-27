import express from 'express'
import cors from 'cors'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import fs from 'fs/promises'
import path from 'path'
import { exec } from 'child_process'
import { promisify } from 'util'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const execAsync = promisify(exec)

const app = express()
const PORT = process.env.PORT || 5000

// Middleware
app.use(cors())
app.use(express.json({ limit: '50mb' }))

// Рабочая директория проекта (можно настроить)
const WORKSPACE_ROOT = process.env.WORKSPACE_ROOT || process.cwd()

// ==================== FILE SYSTEM API ====================

// Получить дерево файлов
app.get('/api/files/tree', async (req, res) => {
  try {
    const requestPath = req.query.path || '/'
    const fullPath = requestPath === '/' ? WORKSPACE_ROOT : join(WORKSPACE_ROOT, requestPath)
    
    // Защита от выхода за пределы рабочей директории
    if (!fullPath.startsWith(WORKSPACE_ROOT)) {
      return res.status(403).json({ error: 'Доступ запрещён' })
    }

    const items = await fs.readdir(fullPath, { withFileTypes: true })
    const tree = []

    for (const item of items) {
      // Пропускаем скрытые файлы и node_modules
      if (item.name.startsWith('.') || item.name === 'node_modules') {
        continue
      }

      if (item.isDirectory()) {
        tree.push({
          name: item.name,
          type: 'directory',
          path: join(requestPath, item.name).replace(/\\/g, '/')
        })
      } else {
        tree.push({
          name: item.name,
          type: 'file',
          path: join(requestPath, item.name).replace(/\\/g, '/'),
          size: (await fs.stat(join(fullPath, item.name))).size
        })
      }
    }

    res.json(tree)
  } catch (error) {
    console.error('Error reading directory:', error)
    res.status(500).json({ error: error.message })
  }
})

// Читать файл
app.post('/api/files/read', async (req, res) => {
  try {
    const { path: filePath } = req.body
    const fullPath = join(WORKSPACE_ROOT, filePath)

    if (!fullPath.startsWith(WORKSPACE_ROOT)) {
      return res.status(403).json({ error: 'Доступ запрещён' })
    }

    const content = await fs.readFile(fullPath, 'utf-8')
    res.json({ content, path: filePath })
  } catch (error) {
    console.error('Error reading file:', error)
    res.status(500).json({ error: error.message })
  }
})

// Записать файл
app.post('/api/files/write', async (req, res) => {
  try {
    const { path: filePath, content } = req.body
    const fullPath = join(WORKSPACE_ROOT, filePath)

    if (!fullPath.startsWith(WORKSPACE_ROOT)) {
      return res.status(403).json({ error: 'Доступ запрещён' })
    }

    // Создаём директорию, если не существует
    await fs.mkdir(path.dirname(fullPath), { recursive: true })
    await fs.writeFile(fullPath, content, 'utf-8')
    
    res.json({ success: true, path: filePath })
  } catch (error) {
    console.error('Error writing file:', error)
    res.status(500).json({ error: error.message })
  }
})

// Удалить файл
app.post('/api/files/delete', async (req, res) => {
  try {
    const { path: filePath } = req.body
    const fullPath = join(WORKSPACE_ROOT, filePath)

    if (!fullPath.startsWith(WORKSPACE_ROOT)) {
      return res.status(403).json({ error: 'Доступ запрещён' })
    }

    await fs.unlink(fullPath)
    res.json({ success: true, path: filePath })
  } catch (error) {
    console.error('Error deleting file:', error)
    res.status(500).json({ error: error.message })
  }
})

// ==================== CONSOLE API ====================

// Выполнить команду в консоли
app.post('/api/console/execute', async (req, res) => {
  try {
    const { command } = req.body
    
    const { stdout, stderr } = await execAsync(command, {
      cwd: WORKSPACE_ROOT,
      timeout: 30000 // 30 секунд таймаут
    })

    res.json({ stdout, stderr, success: !stderr })
  } catch (error) {
    console.error('Error executing command:', error)
    res.json({ 
      stdout: error.stdout || '', 
      stderr: error.stderr || error.message, 
      success: false 
    })
  }
})

// ==================== CHAT API (LLAMA.CPP INTEGRATION) ====================

// Отправка сообщения нейросети
app.post('/api/chat', async (req, res) => {
  try {
    const { message } = req.body
    
    // Здесь будет интеграция с llama.cpp
    // Пока возвращаем заглушку
    const response = await callLlamaCpp(message)
    
    res.json({ response })
  } catch (error) {
    console.error('Error in chat:', error)
    res.status(500).json({ error: error.message })
  }
})

// Функция вызова llama.cpp с Qwen 2.5 14b
async function callLlamaCpp(prompt) {
  const LLAMA_CPP_PATH = process.env.LLAMA_CPP_PATH || './llama.cpp/main'
  const MODEL_PATH = process.env.MODEL_PATH || './models/qwen-2.5-14b-coder.gguf'
  
  try {
    // Проверка существования модели
    try {
      await fs.access(MODEL_PATH)
    } catch {
      return `[Система] Модель не найдена по пути: ${MODEL_PATH}\n\nДля работы AI необходимо:\n1. Установить llama.cpp\n2. Скачать модель Qwen 2.5 14b Coder в формате GGUF\n3. Настроить переменные окружения LLAMA_CPP_PATH и MODEL_PATH`
    }

    // Формирование промпта для кодера
    const systemPrompt = `Ты - опытный программист-ассистент. Твоя задача - помогать с написанием, анализом и рефакторингом кода. Отвечай кратко и по делу.`
    const fullPrompt = `${systemPrompt}\n\nUser: ${prompt}\nAssistant:`

    // Вызов llama.cpp
    const { stdout } = await execAsync(`
      "${LLAMA_CPP_PATH}" \
        -m "${MODEL_PATH}" \
        -p "${fullPrompt.replace(/"/g, '\\"')}" \
        -n 512 \
        --temp 0.7 \
        --top-p 0.9 \
        -e \
        --color never
    `, { 
      timeout: 60000,
      maxBuffer: 10 * 1024 * 1024
    })

    // Очистка ответа от промпта
    const answer = stdout.split('Assistant:').pop()?.trim() || stdout.trim()
    return answer

  } catch (error) {
    console.error('Llama.cpp error:', error)
    return `[Ошибка AI] Не удалось получить ответ от модели: ${error.message}`
  }
}

// ==================== SERVER START ====================

app.listen(PORT, () => {
  console.log(`🚀 AI Ecosystem Interface Server запущен на порту ${PORT}`)
  console.log(`📁 Workspace: ${WORKSPACE_ROOT}`)
  console.log(`🤖 Model: ${process.env.MODEL_PATH || './models/qwen-2.5-14b-coder.gguf'}`)
})
