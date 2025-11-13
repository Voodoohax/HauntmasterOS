export function useWebSocket() {
  const ws = new WebSocket(`ws://${location.host}/ws`)
  const callbacks = []

  ws.onmessage = (e) => {
    const data = JSON.parse(e.data)
    callbacks.forEach(cb => cb(data))
  }

  return {
    send: (msg) => ws.send(JSON.stringify(msg)),
    onMessage: (cb) => callbacks.push(cb)
  }
}
