export async function sendMessage(message) {
  const response = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message })
  })

  if (!response.ok) {
    const error = await response.json()
    throw new Error(error.error || 'Ошибка отправки сообщения')
  }

  const data = await response.json()
  return data.response
}
