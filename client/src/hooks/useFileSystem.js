import { useState, useCallback } from 'react'

export function useFileSystem() {
  const [files, setFiles] = useState({})
  const [openFiles, setOpenFiles] = useState([])
  const [activeFile, setActiveFile] = useState(null)

  // Загрузка структуры файлов с сервера
  const loadFileTree = useCallback(async (path = '/') => {
    try {
      const response = await fetch(`/api/files/tree?path=${encodeURIComponent(path)}`)
      if (response.ok) {
        return await response.json()
      }
    } catch (error) {
      console.error('Error loading file tree:', error)
    }
    return null
  }, [])

  // Чтение файла
  const readFile = useCallback(async (filePath) => {
    try {
      const response = await fetch(`/api/files/read`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath })
      })
      if (response.ok) {
        const data = await response.json()
        setFiles(prev => ({
          ...prev,
          [filePath]: {
            path: filePath,
            name: filePath.split('/').pop(),
            content: data.content,
            type: 'file'
          }
        }))
        return data.content
      }
    } catch (error) {
      console.error('Error reading file:', error)
    }
    return null
  }, [])

  // Запись файла
  const writeFile = useCallback(async (filePath, content) => {
    try {
      const response = await fetch(`/api/files/write`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath, content })
      })
      return response.ok
    } catch (error) {
      console.error('Error writing file:', error)
    }
    return false
  }, [])

  // Удаление файла
  const deleteFile = useCallback(async (filePath) => {
    try {
      const response = await fetch(`/api/files/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath })
      })
      if (response.ok) {
        setFiles(prev => {
          const newFiles = { ...prev }
          delete newFiles[filePath]
          return newFiles
        })
        setOpenFiles(prev => prev.filter(f => f !== filePath))
        if (activeFile === filePath) {
          setActiveFile(null)
        }
        return true
      }
    } catch (error) {
      console.error('Error deleting file:', error)
    }
    return false
  }, [activeFile])

  // Открытие файла
  const openFile = useCallback(async (filePath) => {
    if (!files[filePath]) {
      await readFile(filePath)
    }
    if (!openFiles.includes(filePath)) {
      setOpenFiles(prev => [...prev, filePath])
    }
    setActiveFile(filePath)
  }, [files, openFiles, readFile])

  // Закрытие файла
  const closeFile = useCallback((filePath) => {
    setOpenFiles(prev => prev.filter(f => f !== filePath))
    if (activeFile === filePath) {
      setActiveFile(openFiles.find(f => f !== filePath) || null)
    }
  }, [activeFile, openFiles])

  // Обновление содержимого файла
  const updateFileContent = useCallback((filePath, content) => {
    setFiles(prev => ({
      ...prev,
      [filePath]: {
        ...prev[filePath],
        content
      }
    }))
  }, [])

  // Сохранение файла
  const saveFile = useCallback(async (filePath) => {
    if (files[filePath]) {
      return await writeFile(filePath, files[filePath].content)
    }
    return false
  }, [files, writeFile])

  return {
    files,
    openFiles,
    activeFile,
    openFile,
    closeFile,
    updateFileContent,
    saveFile,
    deleteFile,
    readFile,
    writeFile,
    loadFileTree
  }
}
