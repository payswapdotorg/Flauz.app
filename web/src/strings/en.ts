// The i18n-ready user-visible string dictionary (WEB-001). Every
// user-visible string flows through t(); the single shipped locale is
// `en`, structured so additional locales slot in without touching
// components.

export type StringKey = keyof typeof en;

export const en = {
  "app.title": "Flauz",
  "app.subtitle": "Your shared agent workspace",
  "banner.connected": "Connected",
  "banner.authenticating": "Signing in",
  "banner.reconnecting": "Reconnecting",
  "banner.failed": "Connection failed",
  "banner.retryNow": "Retry now",
  "banner.nextRetry": "Retrying in {seconds}s…",
  "banner.stateDetail": "What happened: {reason}",
  "banner.session": "Session {id}",
  "signin.title": "Sign in to Flauz",
  "signin.description":
    "Flauz connects to your agent runtime through the local gateway. Sign in with your ChatGPT account or an API key.",
  "signin.chatgpt.button": "Sign in with ChatGPT",
  "signin.chatgpt.pending": "Waiting for you to finish signing in…",
  "signin.chatgpt.open": "Open the authorization page",
  "signin.chatgpt.copied": "Authorization link copied",
  "signin.chatgpt.copy": "Copy authorization link",
  "signin.chatgpt.cancel": "Cancel sign-in",
  "signin.apikey.label": "API key",
  "signin.apikey.button": "Sign in with API key",
  "signin.apikey.hint": "Your key is sent only to your local agent runtime and is never logged or stored by Flauz.",
  "signin.error": "Sign-in failed: {reason}",
  "signin.waitingRuntime": "Connecting to your agent runtime before sign-in…",
  "workspace.title": "Workspace",
  "workspace.sessions.heading": "Sessions",
  "workspace.sessions.description": "Tasks you or your agents are working on, most recent first.",
  "workspace.newTask.button": "Start a new task",
  "workspace.empty.title": "Start your first task",
  "workspace.empty.description":
    "Describe what you want in plain language — no jargon needed. Flauz prepares the environment and agents for you.",
  "workspace.empty.action": "Start a new task",
  "workspace.empty.secondary":
    "Prefer to look around first? The command palette (Ctrl+K) lists everything you can do here.",
  "workspace.loadError": "Could not load your sessions: {reason}",
  "workspace.retryLoad": "Try again",
  "workspace.session.open": "Open session",
  "workspace.session.updated": "Updated {when}",
  "newtask.title": "Start a new task",
  "newtask.description": "Describe your objective in ordinary language. You can refine it after you start.",
  "newtask.objective.label": "What do you want to do?",
  "newtask.objective.placeholder": "For example: Plan the community garden layout for next spring",
  "newtask.start": "Start task",
  "newtask.cancel": "Cancel",
  "newtask.error": "Could not start the task: {reason}",
  "session.back": "Back to workspace",
  "session.connecting": "Connecting to session…",
  "session.connected": "Connected — this session is live",
  "session.nextStep": "Describe the next step, or review what happened below.",
  "session.timeline": "Activity",
  "session.timeline.empty": "Nothing has happened in this session yet. Start by describing your objective.",
  "session.composer.label": "Describe the next step",
  "session.composer.placeholder": "Describe the next step for your agents…",
  "session.composer.send": "Send",
  "session.composer.hint": "Sent to your agent runtime; model and environment controls arrive with the capability surfaces.",
  "session.turn.running": "Working…",
  "session.turn.completed": "Completed",
  "session.turn.failed": "Failed: {reason}",
  "context.button": "Context",
  "context.title": "What your agent knows",
  "context.description": "The current objective and activity your agent is working from.",
  "context.objective": "Objective",
  "context.activity": "Recent activity",
  "context.empty": "No context yet — describe your objective to begin.",
  "context.nextStep": "Full context compilation and pinning arrive with the web capability surfaces.",
  "environments.button": "Environments",
  "environments.title": "Environments",
  "environments.empty.title": "No environments attached",
  "environments.empty.description":
    "Attach a terminal, browser, sandbox or desktop so your agents can act. Adding environments arrives with the web capability surfaces.",
  "environments.nextStep": "Ask your agent to suggest where the work should run.",
  "approvals.title": "Needs your decision",
  "approvals.command": "Run this command?",
  "approvals.fileChange": "Apply this file change?",
  "approvals.permissions": "Change permissions?",
  "approvals.approve": "Approve",
  "approvals.approveForSession": "Approve for this session",
  "approvals.decline": "Decline",
  "approvals.declined": "Declined",
  "approvals.approved": "Approved",
  "approvals.permissions.declined":
    "Declined — permission profile selection arrives with the web capability surfaces.",
  "palette.open": "Command palette",
  "palette.placeholder": "Type a command or search…",
  "palette.empty": "No matching commands.",
  "palette.hint": "↑↓ to move · Enter to run · Esc to close",
  "palette.command.home": "Go to workspace",
  "palette.command.newtask": "Start a new task",
  "palette.command.theme": "Toggle light/dark theme",
  "palette.command.reconnect": "Reconnect now",
  "palette.command.signout": "Sign out",
  "palette.command.shortcuts": "Show keyboard shortcuts",
  "shortcuts.title": "Keyboard shortcuts",
  "shortcuts.palette": "Command palette",
  "shortcuts.shortcuts": "This shortcut sheet",
  "shortcuts.close": "Close",
  "shortcuts.escape": "Close overlays and restore focus",
  "theme.toggle": "Toggle theme",
  "signout.button": "Sign out",
  "status.dropped": "Some live updates were delayed — reconnecting to refresh",
  "footer.product": "Flauz — the shared agent workspace",
  "footer.gatewayLocal": "Local gateway",
  "footer.gatewayRemote": "Remote gateway",
} as const;

const dictionaries: Record<string, Record<StringKey, string>> = {
  en,
};

export type Translator = (key: StringKey, values?: Record<string, string | number>) => string;

export function createTranslator(locale = "en"): Translator {
  const dictionary = dictionaries[locale] ?? dictionaries["en"];
  return (key, values) => {
    const template = dictionary[key];
    if (values === undefined) {
      return template;
    }
    return template.replace(/\{(\w+)\}/g, (match, name: string) =>
      name in values ? String(values[name]) : match,
    );
  };
}

export function availableLocales(): string[] {
  return Object.keys(dictionaries);
}
