import { invoke } from "@tauri-apps/api/core"

type ModelSettings = {
  providerID: string
  modelID: string
  apiKey: string
}

function resolveModelSettings(): ModelSettings {
  const stored = localStorage.getItem("model-settings")
  if (stored) {
    try {
      return JSON.parse(stored)
    } catch {}
  }
  return { providerID: "opencode", modelID: "big-pickle", apiKey: "" }
}

type SendMessageOptions = {
  onThinking?: (text: string) => void
  onText?: (text: string) => void
  onEvent?: (event: any) => void
}

class OpencodeService {
  static sessionId = ""
  static baseUrl = ""

  static getBaseUrl() {
    return OpencodeService.baseUrl
  }

  static async initialize(workspace: string) {
    const url: string = await invoke("execute_opencode_serve", { workspace })
    OpencodeService.baseUrl = url
    await OpencodeService.newSession()
  }

  static async sendMessage(message: string, options?: SendMessageOptions) {
    const { onThinking, onText, onEvent } = options || {}
    const modelSettings = resolveModelSettings()

    const shouldSubscribe = onThinking || onText
    let markEventReady: () => void
    const eventReady = shouldSubscribe
      ? new Promise<void>((resolve) => { markEventReady = resolve })
      : Promise.resolve()

    const eventPromise = shouldSubscribe
      ? OpencodeService.subscribeEvents(onEvent, markEventReady!)
      : Promise.resolve()

    await Promise.race([eventReady!, new Promise((r) => setTimeout(r, 1000))])

    try {
      const res = await fetch(
        `${OpencodeService.baseUrl}/session/${OpencodeService.sessionId}/message`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            agent: "build",
            model: {
              modelID: modelSettings.modelID,
              providerID: modelSettings.providerID,
            },
            parts: [{ type: "text", text: message }],
          }),
        },
      )
      const data = await res.json()
      await eventPromise
      return data?.parts?.find((p: any) => p.type === "text")?.text || ""
    } catch (e) {
      await eventPromise
      throw e
    }
  }

  private static async newSession() {
    const res = await fetch(`${OpencodeService.baseUrl}/session`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    })
    const data = await res.json()
    OpencodeService.sessionId = data.id || ""
  }

  private static async subscribeEvents(
    onEvent?: (event: any) => void,
    onOpen?: () => void,
  ) {
    const eventSource = new EventSource(`${OpencodeService.baseUrl}/event`)
    eventSource.onopen = () => onOpen?.()
    eventSource.onmessage = (e) => {
      try {
        const payload = JSON.parse(e.data)?.payload || JSON.parse(e.data)
        onEvent?.(payload)
      } catch {}
    }
    await new Promise<void>((resolve) => {
      eventSource.onopen = () => {
        onOpen?.()
        resolve()
      }
    })
  }

  static async killAll() {
    await invoke("kill_existing_opencode_processes")
    OpencodeService.baseUrl = ""
    OpencodeService.sessionId = ""
  }
}

export default OpencodeService
