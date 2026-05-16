/**
 * Transport abstraction layer — unified WebSocket-only architecture.
 *
 * In the new architecture:
 * - All business logic (chat, sessions, agents, etc.) goes through WebSocket
 * - Only local file operations use Tauri IPC (upload, export, etc.)
 * - Tauri app connects to a Gateway daemon process via WebSocket
 */

import * as wsClient from "./ws-client";

export const isTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

let _invoke: typeof import("@tauri-apps/api/core").invoke | null = null;

async function ensureTauriApi() {
  if (!_invoke) {
    const core = await import("@tauri-apps/api/core");
    _invoke = core.invoke;
  }
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  await ensureTauriApi();
  return _invoke!<T>(cmd, args);
}

// ─── Gateway Info (IPC) ───

export interface GatewayInfo {
  port: number;
  wsUrl: string;
  httpUrl: string;
  version: string;
}

export async function getGatewayInfo(): Promise<GatewayInfo | null> {
  if (!isTauri) return null;
  try {
    return await tauriInvoke<GatewayInfo>("get_gateway_info");
  } catch {
    return null;
  }
}

// ─── Local File Operations (IPC only) ───

export async function uploadAgentAvatar(
  agentId: string,
  sourcePath: string,
): Promise<{ ok: boolean; path?: string }> {
  if (!isTauri) return { ok: false };
  return tauriInvoke("upload_agent_avatar", { agentId, sourcePath });
}

export async function readIdentityFiles(
  agentId: string,
): Promise<{ soul: string | null; user: string | null; agents: string | null; tools: string | null }> {
  if (!isTauri) return { soul: null, user: null, agents: null, tools: null };
  return tauriInvoke("read_identity_files", { agentId });
}

export async function uploadSkill(
  sourcePath: string,
): Promise<{ installed?: string }> {
  if (!isTauri) return {};
  return tauriInvoke("upload_skill", { sourcePath });
}

export interface ExportOptions {
  includeSessions: boolean;
  includeSkills: boolean;
  includeAgentWorkspaces: boolean;
}

export interface ImportOptions {
  merge: boolean;
  overwriteConfig: boolean;
  overwriteAgents: boolean;
  overwriteSessions: boolean;
  overwriteSkills: boolean;
}

export async function exportData(options: ExportOptions): Promise<Uint8Array> {
  if (!isTauri) throw new Error("export only available in desktop mode");
  const result = await tauriInvoke<number[]>("export_data", { options });
  return new Uint8Array(result);
}

export async function importData(data: Uint8Array, options: ImportOptions): Promise<void> {
  if (!isTauri) throw new Error("import only available in desktop mode");
  await tauriInvoke("import_data", {
    data: Array.from(data),
    options,
  });
}

// ─── Session Export (IPC only) ───
// Frontend fetches session content via WebSocket, then passes to IPC for local file save.

export type ExportFormat = "markdown" | "json";

export async function exportSessionContent(
  content: string,
  filename: string,
  mimeType: string,
): Promise<{ success: boolean; path?: string }> {
  if (!isTauri) throw new Error("export only available in desktop mode");
  return tauriInvoke("export_session_content", { content, filename, mimeType });
}

// ─── WebSocket Connection ───

export function connectWs(url: string, token?: string): Promise<void> {
  return wsClient.connect(url, token).then(() => {
    wsClient.send("subscribe", {
      events: [
        "sessions.changed",
        "channels.changed",
        "cron.job.complete",
        "cron.job.failed",
        "notification.new",
        "notification.read",
      ],
    }).catch(() => {});
  });
}

export function disconnectWs(): void {
  wsClient.disconnect();
}

export function isWsConnected(): boolean {
  return wsClient.isConnected();
}

// ─── WebSocket Operations (all business logic) ───

export interface AgentSummary {
  agentId: string;
  name: string;
  model: string;
  avatar?: string | null;
}

export async function listAgents(): Promise<AgentSummary[]> {
  const resp = (await wsClient.send("agents")) as {
    data?: { agents?: AgentSummary[] };
  };
  return resp?.data?.agents ?? [];
}

export interface SessionSummary {
  id: string;
  agentId: string;
  title: string | null;
  workDir?: string | null;
  source?: string;
  messageCount: number;
  createdAt: string;
  updatedAt: string;
  totalPromptTokens?: number;
  totalCompletionTokens?: number;
  totalElapsedMs?: number;
}

export async function listSessions(limit = 50, offset = 0): Promise<SessionSummary[]> {
  const resp = (await wsClient.send("sessions.list", { limit, offset })) as {
    data?: { sessions?: SessionSummary[] };
  };
  return resp?.data?.sessions ?? [];
}

export async function getSession(sessionId: string): Promise<SessionSummary | null> {
  const resp = (await wsClient.send("sessions.get", { sessionId })) as {
    data?: SessionSummary;
  };
  return resp?.data ?? null;
}

export interface SessionMessage {
  id: number;
  role: string;
  content: unknown;
  name: string | null;
  toolCallId: string | null;
  createdAt: string;
}

export async function getSessionMessages(sessionId: string): Promise<SessionMessage[]> {
  const resp = (await wsClient.send("sessions.messages", { sessionId })) as {
    data?: { messages?: SessionMessage[] };
  };
  return resp?.data?.messages ?? [];
}

export async function createSession(agentId?: string): Promise<string> {
  const resp = (await wsClient.send("sessions.new", agentId ? { agentId } : {})) as {
    data?: { sessionId?: string };
  };
  return resp?.data?.sessionId ?? "";
}

export async function updateSessionTitle(sessionId: string, title: string): Promise<void> {
  await wsClient.send("sessions.update_title", { sessionId, title });
}

export async function deleteSession(sessionId: string): Promise<void> {
  await wsClient.send("sessions.delete", { sessionId });
}

export async function setSessionWorkDir(sessionId: string, workDir: string | null): Promise<void> {
  await wsClient.send("sessions.set_work_dir", { sessionId, workDir });
}

export interface ModelInfo {
  agentId: string;
  model: string;
  provider: string;
  contextWindow: number;
  costPer1kInput: number;
  costPer1kOutput: number;
  supportsReasoning: boolean;
  capabilities?: import("./model-registry").ModelCapabilities;
}

export async function listModels(): Promise<ModelInfo[]> {
  const resp = (await wsClient.send("models.list")) as {
    data?: { models?: ModelInfo[] };
  };
  return resp?.data?.models ?? [];
}

export interface SkillInfo {
  id: string;
  name: string;
  description: string | null;
  tags?: string[];
}

export async function listSkills(agentId?: string): Promise<SkillInfo[]> {
  const resp = (await wsClient.send("skills.list", agentId ? { agentId } : {})) as {
    data?: { skills?: SkillInfo[] };
  };
  return resp?.data?.skills ?? [];
}

export async function refreshSkills(): Promise<{ refreshed: boolean; count: number }> {
  const resp = (await wsClient.send("skills.refresh")) as {
    data?: { refreshed?: boolean; count?: number };
  };
  return { refreshed: resp?.data?.refreshed ?? false, count: resp?.data?.count ?? 0 };
}

export async function getAgent(agentId: string): Promise<unknown> {
  const resp = (await wsClient.send("agents.get", { agentId })) as {
    data?: unknown;
  };
  return resp?.data ?? null;
}

export async function createAgent(config: Record<string, unknown>): Promise<{ ok: boolean; agentId?: string }> {
  const resp = (await wsClient.send("agents.create", { config })) as {
    data?: { ok?: boolean; agentId?: string };
  };
  return { ok: resp?.data?.ok ?? false, agentId: resp?.data?.agentId };
}

export async function updateAgent(agentId: string, config: Record<string, unknown>): Promise<boolean> {
  const resp = (await wsClient.send("agents.update", { agentId, config })) as {
    data?: { ok?: boolean };
  };
  return resp?.data?.ok ?? false;
}

export async function deleteAgent(agentId: string): Promise<boolean> {
  const resp = (await wsClient.send("agents.delete", { agentId })) as {
    data?: { ok?: boolean };
  };
  return resp?.data?.ok ?? false;
}

export async function getConfig(key?: string): Promise<unknown> {
  const resp = (await wsClient.send("config.get", key ? { key } : {})) as {
    data?: unknown;
  };
  return resp?.data;
}

export async function setConfig(
  key: string,
  value: unknown,
): Promise<{ persisted: boolean; pendingRestart: boolean }> {
  const resp = (await wsClient.send("config.set", { key, value })) as {
    data?: { persisted?: boolean; pendingRestart?: boolean };
  };
  return {
    persisted: resp?.data?.persisted ?? false,
    pendingRestart: resp?.data?.pendingRestart ?? false,
  };
}

// ─── Chat Streaming (WebSocket) ───

export interface ChatStreamEvent {
  type: string;
  data?: Record<string, unknown>;
  error?: { message: string };
}

type ChatEventHandler = (event: ChatStreamEvent) => void;

export interface ChatStreamParams {
  messages: Array<{ role: string; content: string | unknown[] }>;
  agentId?: string;
  sessionId?: string;
  model?: string;
  temperature?: number;
  maxTokens?: number;
  workDir?: string;
}

export function chatStream(
  params: ChatStreamParams,
  onEvent: ChatEventHandler,
): { promise: Promise<void>; cleanup: () => void } {
  const handlers: Array<(() => void) | undefined> = [];
  let done = false;

  const wrap = (type: string) => {
    const unsub = wsClient.on(type, (m: unknown) => {
      if (!done) onEvent(m as ChatStreamEvent);
    });
    handlers.push(unsub);
  };

  wrap("chat.start");
  wrap("chat.delta");
  wrap("chat.complete");
  wrap("chat.tool.start");
  wrap("chat.tool.done");
  wrap("chat.ask_question");
  wrap("chat.error");

  const cleanup = () => {
    done = true;
    handlers.forEach((h) => {
      if (typeof h === "function") h();
    });
  };

  const promise = wsClient
    .send("chat", {
      messages: params.messages,
      agentId: params.agentId,
      sessionId: params.sessionId,
      stream: true,
      ...(params.workDir ? { workDir: params.workDir } : {}),
    })
    .then(() => {})
    .catch(() => {
      cleanup();
    });

  return { promise, cleanup };
}

// ─── Event Subscriptions ───

export type UnsubscribeFn = () => void;

export function onSessionChanged(handler: (sessionId: string) => void): UnsubscribeFn {
  return wsClient.on("sessions.changed", (msg: unknown) => {
    const sid = (msg as { data?: { sessionId?: string } })?.data?.sessionId;
    if (sid) handler(sid);
  });
}

export function onChannelsChanged(handler: (channelId: string, action: string) => void): UnsubscribeFn {
  return wsClient.on("channels.changed", (msg: unknown) => {
    const data = (msg as { data?: { channelId?: string; action?: string } })?.data;
    if (data?.channelId) handler(data.channelId, data.action ?? "updated");
  });
}

export function onWsEvent(event: string, handler: (data: unknown) => void): UnsubscribeFn {
  return wsClient.on(event, handler);
}

// ─── MCP Server Management ───

export interface McpServerStatus {
  id: string;
  status: "connecting" | "connected" | "failed" | "disabled";
  error?: string | null;
  toolCount: number;
  connectedAt?: string | null;
}

export async function getMcpStatus(): Promise<McpServerStatus[]> {
  const resp = await wsClient.send("mcp.status");
  return (resp as { servers?: McpServerStatus[] }).servers ?? [];
}

export async function reloadMcpServers(): Promise<McpServerStatus[]> {
  const resp = await wsClient.send("mcp.reload");
  return (resp as { servers?: McpServerStatus[] }).servers ?? [];
}

export async function addMcpServer(
  id: string,
  command: string,
  args?: string[],
): Promise<{ ok: boolean; id: string; status?: McpServerStatus }> {
  return wsClient.send("mcp.add", { id, command, args: args ?? [] }) as Promise<{
    ok: boolean;
    id: string;
    status?: McpServerStatus;
  }>;
}

export async function removeMcpServer(id: string): Promise<{ ok: boolean; id: string }> {
  return wsClient.send("mcp.remove", { id }) as Promise<{
    ok: boolean;
    id: string;
  }>;
}

// ─── Tools ───

export interface AgentToolInfo {
id: string;
  enabled: boolean;
  description?: string;
}

export async function listAgentTools(agentId: string): Promise<AgentToolInfo[]> {
  const resp = (await wsClient.send("tools.list", { agentId })) as {
    data?: { tools?: AgentToolInfo[] };
  };
  return resp?.data?.tools ?? [];
}

export async function updateAgentTools(
  agentId: string,
  tools: Array<{ id: string; enabled: boolean }>,
): Promise<boolean> {
  const resp = (await wsClient.send("tools.update", { agentId, tools })) as {
    data?: { ok?: boolean };
  };
  return resp?.data?.ok ?? false;
}

// ─── Execution Mode ───

export async function setExecutionMode(
  mode: "agent" | "plan",
  sessionId?: string,
): Promise<{ ok: boolean; from: string; to: string }> {
  const resp = (await wsClient.send("execution.set_mode", { mode, sessionId })) as {
    data?: { ok?: boolean; from?: string; to?: string };
  };
  return {
    ok: resp?.data?.ok ?? false,
    from: resp?.data?.from ?? "",
    to: resp?.data?.to ?? "",
  };
}

export async function getPlanFile(sessionId?: string): Promise<{ path: string; content: string | null; exists: boolean }> {
  const resp = (await wsClient.send("execution.get_plan", { sessionId })) as {
    data?: { path?: string; content?: string | null; exists?: boolean };
  };
  return {
    path: resp?.data?.path ?? "",
    content: resp?.data?.content ?? null,
    exists: resp?.data?.exists ?? false,
  };
}

export async function submitToolAnswer(requestId: string, answer: string): Promise<{ ok: boolean }> {
  const resp = (await wsClient.send("tools.submit_answer", { requestId, answer })) as {
    data?: { ok?: boolean };
  };
  return { ok: resp?.data?.ok ?? false };
}

// ─── Backward Compatibility Aliases ───
// These are kept for gradual migration of the frontend code.

export const listSkillsIpc = listSkills;
export const uploadSkillIpc = uploadSkill;
export const listAgentToolsIpc = listAgentTools;
export const updateAgentToolsIpc = updateAgentTools;
export const refreshSkillsIpc = refreshSkills;
export const submitToolAnswerIpc = submitToolAnswer;
export const setExecutionModeIpc = setExecutionMode;
export const getPlanFileIpc = getPlanFile;
export const getAgentIpc = getAgent;
export const updateAgentIpc = async (agentId: string, config: Record<string, unknown>) => updateAgent(agentId, config);
export const createAgentIpc = createAgent;
export const deleteAgentIpc = async (agentId: string) => deleteAgent(agentId);
export const uploadAgentAvatarIpc = uploadAgentAvatar;
export const readIdentityFilesIpc = readIdentityFiles;
export const reloadChannelIpc = async (_channelId: string) => false; // deprecated