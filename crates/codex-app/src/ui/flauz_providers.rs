//! The provider-accounts surface (work order PROV-001, F7 Wave 4) — GUI
//! module: BYOP connections, quota attribution in plain words, and the
//! free-tier-first routing policy the user can see and change.
//!
//! Seven discoverability layers (PRODUCT-UX-JOURNEYS §1 / the shell-family
//! discipline):
//!
//! 1. **Visible labeled control** — a `Provider accounts` control on the
//!    task surface, showing the connected-account count (or the honest
//!    "none yet") — never hidden behind developer settings.
//! 2. **Contextual affordance** — the task-surface attribution line:
//!    "Using your OpenAI free tier — 3 of 5 runs left today", and the
//!    depletion moment: "Your free tier is used up today — next run uses
//!    your paid account. Change this" (the "Change this" action opens the
//!    routing-policy section).
//! 3. **Palette rows** — "Connect a provider account" and "See which
//!    account a task uses".
//! 4. **Keyboard path** — `Ctrl+Alt+Shift+P` opens the surface; Escape
//!    closes it through the scoped focus context (never a trap).
//! 5. **Honest empty state** — no accounts connected: what connecting
//!    adds, and the connect form as the next step.
//! 6. **Success state** — after connecting: the account appears with its
//!    tier and quota state in user words, plus the honest "The key is
//!    stored securely; Flauz never shows it again" line.
//! 7. **Truthful failure states** — the connect form refuses an empty key
//!    and refuses material that looks like a real credential (this
//!    build's flows run on practice keys only — nothing was stored);
//!    depleted accounts say so plainly.
//!
//! Boundary rules (work order PROV-001 / kernel §1):
//!
//! - the app crate does NOT import the `flauz-prov` contract crate in
//!   this wave — the **attribution view-model seam**
//!   ([`TaskAttributionView`]) and the **connect-flow fake seam**
//!   ([`connect_account_through_fake_seam`]) below are the minimal,
//!   unit-testable shapes a later slice wires to the real scheduler and
//!   the real secret store; the seams keep the frozen vocabulary (the
//!   `flausec_` reference prefix as data; the tier words; the free-tier
//!   -first default);
//! - credentials are references, forever (addendum §3): the connect flow
//!   moves the entered key through the fake seam, which mints a
//!   `flausec_` reference and retains NOTHING — the account view holds
//!   the reference (data, never rendered) and the "stored securely" line
//!   instead of the key;
//! - existing F1 flows are untouched: the module is wired through
//!   minimal `ui.rs` named seams (module declaration, palette rows,
//!   keyboard chord, scoped escape, state field, task-surface mount,
//!   navigation close) — distinct from every prior wave's seams;
//! - the F1 focus contracts are followed (the picker/save-flow shape):
//!   capture the previously focused surface on open, auto-focus the
//!   panel's handle once, restore on close.
//!
//! Every user-facing string lives in the copy registry below so the
//! language rules (money/quota in plain words; never "flausec_", never
//! "token budget", never internal type names) are unit-testable in one
//! place.

use codex_core::MainRoute;
use gpui::prelude::*;
use gpui::{AnyElement, Context, Entity, FocusHandle, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Selectable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Escape, Input, InputState},
    v_flex,
};

use super::WorkspaceView;

gpui::actions!(codexrs, [FlauzProvidersShortcut]);

// ---------------------------------------------------------------------------
// Copy registry (user language; money/quota in plain words — addendum §7)
// ---------------------------------------------------------------------------

/// The labeled control's noun (layer 1).
pub(crate) const ENTRY_LABEL: &str = "Provider accounts";

/// The control's tooltip (the chord label matches the keyboard
/// registration).
pub(crate) const ENTRY_TOOLTIP: &str = "Connect your own provider accounts and choose which \
     one Flauz uses (Ctrl+Alt+Shift+P)";

/// The control's summary when no account is connected (honest, never
/// implying a connection exists).
pub(crate) const ENTRY_SUMMARY_EMPTY: &str = "None connected yet";

/// The control's summary shape when accounts are connected: "2 accounts \
/// · free accounts first" (the routing order in plain words).
pub(crate) const ENTRY_SUMMARY_SHAPE: &str = "{count} connected · {order}";

/// The command-palette row title for the connect flow (layer 3 — the
/// work order's exact row).
pub(crate) const PALETTE_ROW_CONNECT_TITLE: &str = "Connect a provider account";

/// The connect row's description.
pub(crate) const PALETTE_ROW_CONNECT_DESCRIPTION: &str =
    "Add your own account for a provider, with its own quota and limits";

/// The command-palette row title for the attribution affordance (layer 3
/// — the work order's exact row).
pub(crate) const PALETTE_ROW_SEE_TITLE: &str = "See which account a task uses";

/// The see row's description.
pub(crate) const PALETTE_ROW_SEE_DESCRIPTION: &str =
    "Show whose account a task draws on, which tier, and what is left";

/// The panel heading (layer 2).
pub(crate) const PANEL_HEADING: &str = "Your provider accounts";

/// The one-line description under the panel heading.
pub(crate) const PANEL_DESCRIPTION: &str = "Accounts you connect let Flauz run tasks with \
     your provider quota. Flauz says whose account a task uses, and tells you before it \
     uses a paid account.";

/// The honest empty-state title (layer 5).
pub(crate) const EMPTY_TITLE: &str = "No provider accounts connected yet";

/// The honest empty-state body: what connecting adds.
pub(crate) const EMPTY_BODY: &str = "Connect an account you already have — for example an \
     OpenAI account — and Flauz can run tasks with your quota instead of a shared one. \
     Flauz shows what is left before each run, and never spends a paid account without \
     saying so.";

/// The connect form's heading.
pub(crate) const CONNECT_HEADING: &str = "Connect a provider account";

/// The provider label shown on the form (the fake provider this build's
/// flow connects to).
pub(crate) const CONNECT_PROVIDER_LABEL: &str = "OpenAI";

/// The account-name entry's label.
pub(crate) const CONNECT_NAME_LABEL: &str = "Account name";

/// The account-name entry's placeholder.
pub(crate) const CONNECT_NAME_PLACEHOLDER: &str = "For example: Personal account";

/// The API-key entry's label.
pub(crate) const CONNECT_KEY_LABEL: &str = "API key";

/// The API-key entry's placeholder.
pub(crate) const CONNECT_KEY_PLACEHOLDER: &str = "Paste the key from your provider";

/// The connect action's label.
pub(crate) const CONNECT_BUTTON: &str = "Connect account";

/// The honest storage line after connecting (the work order's exact
/// state): the key is gone from the surface, forever.
pub(crate) const CONNECT_SECURE_NOTE: &str =
    "The key is stored securely; Flauz never shows it again.";

/// The truthful failure when the key entry is empty.
pub(crate) const CONNECT_EMPTY_KEY_FAILURE: &str =
    "Enter the key you copied from your provider, then connect.";

/// The truthful failure when the entered material looks like a real
/// credential: this build's flows run on practice keys only, and nothing
/// was stored.
pub(crate) const CONNECT_LOOKS_REAL_FAILURE: &str = "This looks like a real key. This build only accepts clearly fake practice keys — \
     nothing was stored.";

/// The truthful failure when the account name is empty.
pub(crate) const CONNECT_EMPTY_NAME_FAILURE: &str = "Give this account a name first.";

/// The accounts list caption.
pub(crate) const ACCOUNTS_LIST_CAPTION: &str = "Connected accounts";

/// The tier chip for a free account.
pub(crate) const TIER_FREE_LABEL: &str = "Free tier";

/// The tier chip for a paid account.
pub(crate) const TIER_PAID_LABEL: &str = "Paid account";

/// The quota line shape in user words: "3 of 5 runs left today" (the
/// work order's exact style).
pub(crate) const QUOTA_LINE_SHAPE: &str = "{remaining} of {limit} runs left {window}";

/// The window word for daily windows.
pub(crate) const WINDOW_WORD_TODAY: &str = "today";

/// The depleted quota line.
pub(crate) const QUOTA_DEPLETED_LINE: &str = "Used up for today";

/// The per-account storage line (shown on every account row).
pub(crate) const ACCOUNT_SECURE_NOTE: &str = "Stored securely — Flauz never shows it again.";

/// The routing-policy section heading.
pub(crate) const POLICY_HEADING: &str = "Which account Flauz uses";

/// The policy order line shape: the current order in plain words.
pub(crate) const POLICY_ORDER_SHAPE: &str = "Order today: {order}";

/// The free-first order's plain-words label.
pub(crate) const ORDER_FREE_LABEL: &str = "free accounts first";

/// The paid-first order's plain-words label.
pub(crate) const ORDER_PAID_LABEL: &str = "paid accounts first";

/// The change action's label.
pub(crate) const POLICY_CHANGE_BUTTON: &str = "Change the order";

/// The consequence of switching to paid-first, stated plainly (the
/// surface always states consequences).
pub(crate) const POLICY_CONSEQUENCE_TO_PAID: &str = "Switched to paid accounts first: tasks will use your paid accounts even when free \
     quota is left.";

/// The consequence of switching back to free-first.
pub(crate) const POLICY_CONSEQUENCE_TO_FREE: &str = "Switched to free accounts first: Flauz uses free quota before paid accounts, and \
     tells you before it uses a paid account.";

/// The spend and concurrency limits caption.
pub(crate) const LIMITS_CAPTION: &str = "Spending and task limits";

/// The per-account spend limit line shape.
pub(crate) const ACCOUNT_SPEND_SHAPE: &str = "Up to {limit} uses per account each day";

/// The workspace spend limit line shape.
pub(crate) const WORKSPACE_SPEND_SHAPE: &str = "Up to {limit} uses across this workspace each day";

/// The per-account concurrency limit line shape.
pub(crate) const ACCOUNT_CONCURRENCY_SHAPE: &str = "At most {limit} tasks at a time per account";

/// The workspace concurrency limit line shape.
pub(crate) const WORKSPACE_CONCURRENCY_SHAPE: &str =
    "At most {limit} tasks at a time in this workspace";

/// The honest no-limits fallback.
pub(crate) const NO_LIMITS_LINE: &str =
    "No limits set — Flauz will tell you before a task uses a paid account.";

/// The task-surface attribution line shape (layer 2, the work order's
/// exact style): "Using your OpenAI free tier — 3 of 5 runs left today".
pub(crate) const ATTRIBUTION_LINE_SHAPE: &str =
    "Using your {provider} {tier} — {remaining} of {limit} runs left {window}";

/// The depletion moment (the work order's exact phrase): the free tier
/// is used up, the next run will use the paid account, and the change
/// action is offered.
pub(crate) const DEPLETION_LINE: &str =
    "Your free tier is used up today — next run uses your paid account. Change this";

/// The "Change this" action on the depletion moment (opens the
/// routing-policy section).
pub(crate) const DEPLETION_CHANGE_ACTION: &str = "Change this";

/// The see-row guidance when no chat is selected (guidance, never a
/// silent no-op — the WO-P2-012 pattern).
pub(crate) const SEE_NO_CHAT_GUIDANCE: &str = "Open a chat to see which account it uses.";

/// The see-row guidance when the selected task has no attribution yet
/// (the wiring seam has not attached one — the honest not-yet state).
pub(crate) const SEE_NO_ATTRIBUTION_GUIDANCE: &str =
    "This task has not been set up with an account yet — it will get one when it next runs.";

/// The panel footer hint (the keyboard close path).
pub(crate) const PROVIDERS_ESCAPE_HINT: &str = "Escape closes this panel";

// ---------------------------------------------------------------------------
// The view-model (plain data; the wiring seam populates it)
// ---------------------------------------------------------------------------

/// The tier vocabulary the surface renders (the frozen free/paid words).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the connect-flow fake seam and the module's tests; the wiring seam builds production views
pub(crate) enum TierView {
    /// The account's free quota.
    Free,
    /// A paid account.
    Paid,
}

impl TierView {
    /// The user-facing chip label.
    pub(crate) const fn chip_label(self) -> &'static str {
        match self {
            Self::Free => TIER_FREE_LABEL,
            Self::Paid => TIER_PAID_LABEL,
        }
    }

    /// The tier noun inside the attribution line ("free tier" /
    /// "paid account").
    pub(crate) const fn attribution_noun(self) -> &'static str {
        match self {
            Self::Free => TIER_FREE_LABEL,
            Self::Paid => TIER_PAID_LABEL,
        }
    }
}

/// The account routing order the surface renders (the frozen
/// free-tier-first default as data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // read by the control summary and the policy section; flipped by the change action
pub(crate) enum OrderView {
    /// Free accounts are tried first (the default).
    FreeFirst,
    /// Paid accounts are tried first (the user's explicit change).
    PaidFirst,
}

impl OrderView {
    /// The plain-words label for this order.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::FreeFirst => ORDER_FREE_LABEL,
            Self::PaidFirst => ORDER_PAID_LABEL,
        }
    }

    /// The other order (the change action flips between the two).
    pub(crate) const fn flipped(self) -> Self {
        match self {
            Self::FreeFirst => Self::PaidFirst,
            Self::PaidFirst => Self::FreeFirst,
        }
    }
}

/// One connected account, as the surface renders it: whose account, the
/// provider, the tier, the quota state in user words, and the opaque
/// storage reference (DATA — never rendered; the "stored securely" line
/// is what the user sees instead of the key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderAccountView {
    /// The user-words label for the account ("whose account").
    pub(crate) account_label: String,
    /// The provider's user-facing label.
    pub(crate) provider_label: String,
    /// The account's tier.
    pub(crate) tier: TierView,
    /// How many runs remain in the current window.
    pub(crate) quota_remaining: u32,
    /// The window's run limit.
    pub(crate) quota_limit: u32,
    /// The window word ("today" for daily windows).
    pub(crate) window_word: &'static str,
    /// The opaque storage reference minted at connect time (`flausec_`
    /// prefixed, as data). Never rendered; never credential material.
    #[allow(dead_code)]
    // the storage-law proof: retained as data, never shown — pinned by the module's tests
    pub(crate) secret_reference: String,
}

impl ProviderAccountView {
    /// The quota line in user words ("3 of 5 runs left today"), or the
    /// depleted line when nothing remains.
    pub(crate) fn quota_line(&self) -> String {
        if self.quota_remaining == 0 {
            return QUOTA_DEPLETED_LINE.to_owned();
        }
        QUOTA_LINE_SHAPE
            .replace("{remaining}", &self.quota_remaining.to_string())
            .replace("{limit}", &self.quota_limit.to_string())
            .replace("{window}", self.window_word)
    }

    /// Whether the account's quota is used up.
    #[allow(dead_code)] // pinned by the module's tests; the quota line renders the state directly
    pub(crate) const fn is_depleted(&self) -> bool {
        self.quota_remaining == 0
    }
}

/// The task-surface attribution view-model: which account a task uses or
/// will use, and why — populated by the wiring seam (a later slice wires
/// the real scheduler's choices here); until then there is no line and
/// no invented data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // constructed by the wiring seam and the module's tests (the RunShape precedent)
pub(crate) struct TaskAttributionView {
    /// The chosen account's user-words label.
    #[allow(dead_code)] // rendered through the secondary line; pinned by the module's tests
    pub(crate) account_label: String,
    /// The provider's user-facing label.
    pub(crate) provider_label: String,
    /// The chosen account's tier.
    pub(crate) tier: TierView,
    /// How many runs remain, in the account's window.
    pub(crate) quota_remaining: u32,
    /// The window's run limit.
    pub(crate) quota_limit: u32,
    /// The window word ("today" for daily windows).
    pub(crate) window_word: &'static str,
    /// The plain-words reason the account was chosen (the policy rule's
    /// explanation, user words).
    pub(crate) why: String,
    /// Whether a paid account exists as the next step when this (free)
    /// tier depletes — the depletion moment's "next run uses your paid
    /// account" is only honest when one does.
    pub(crate) paid_account_available: bool,
}

impl TaskAttributionView {
    /// The task-surface attribution line (layer 2, the work order's exact
    /// style): "Using your OpenAI free tier — 3 of 5 runs left today".
    pub(crate) fn line(&self) -> String {
        ATTRIBUTION_LINE_SHAPE
            .replace("{provider}", &self.provider_label)
            .replace("{tier}", self.tier.attribution_noun())
            .replace("{remaining}", &self.quota_remaining.to_string())
            .replace("{limit}", &self.quota_limit.to_string())
            .replace("{window}", self.window_word)
    }

    /// Whether the depletion moment is NOW: the (free) tier in use is
    /// used up and a paid account exists to fall back to.
    pub(crate) fn is_depleted_moment(&self) -> bool {
        self.tier == TierView::Free && self.quota_remaining == 0 && self.paid_account_available
    }
}

// ---------------------------------------------------------------------------
// The connect-flow fake seam (the frozen vocabulary as data)
// ---------------------------------------------------------------------------

/// Why a connect attempt failed — the truthful failure states, each with
/// its user-words line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectFailure {
    /// The key entry was empty.
    EmptyKey,
    /// The account name was empty.
    EmptyName,
    /// The entered material looks like a real credential: this build's
    /// flows accept only clearly fake practice keys, and nothing was
    /// stored.
    LooksReal,
}

impl ConnectFailure {
    /// The truthful user-words line for this failure.
    pub(crate) const fn user_line(self) -> &'static str {
        match self {
            Self::EmptyKey => CONNECT_EMPTY_KEY_FAILURE,
            Self::EmptyName => CONNECT_EMPTY_NAME_FAILURE,
            Self::LooksReal => CONNECT_LOOKS_REAL_FAILURE,
        }
    }
}

/// The marker family the fake seam refuses as real-credential shapes
/// (the frozen vocabulary as data — mirrored from the contract side, so
/// the UI's refusal and the crate's rejection agree).
const REAL_KEY_MARKERS: &[&str] = &[
    "sk-",
    "Bearer ",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "client_secret",
    "access_token",
    "refresh_token",
    "PRIVATE KEY",
    "BEGIN RSA",
    "xoxb-",
    "ghp_",
];

/// The connect-flow fake seam: validates the entered key, mints the
/// opaque storage reference (the `flausec_` prefix as data), and returns
/// the account view that appears — retaining NOTHING of the entered
/// material (the view's only storage trace is the reference string; the
/// "stored securely" line is what the user sees). Deterministic: the
/// Nth connect mints the Nth reference.
///
/// The fake provider this build connects to reports the account as a
/// free-tier account with five runs a day.
pub(crate) fn connect_account_through_fake_seam(
    account_name: &str,
    key_material: &str,
    sequence: u64,
) -> Result<ProviderAccountView, ConnectFailure> {
    if account_name.trim().is_empty() {
        return Err(ConnectFailure::EmptyName);
    }
    if key_material.trim().is_empty() {
        return Err(ConnectFailure::EmptyKey);
    }
    for marker in REAL_KEY_MARKERS {
        if key_material.contains(marker) {
            return Err(ConnectFailure::LooksReal);
        }
    }
    Ok(ProviderAccountView {
        account_label: account_name.trim().to_owned(),
        provider_label: CONNECT_PROVIDER_LABEL.to_owned(),
        tier: TierView::Free,
        quota_remaining: 5,
        quota_limit: 5,
        window_word: WINDOW_WORD_TODAY,
        secret_reference: format!("flausec_practice_{sequence:04}"),
    })
}

// ---------------------------------------------------------------------------
// The providers UI state (additive; the panel wraps the pure views)
// ---------------------------------------------------------------------------

/// The additive providers state: the open panel, the connected accounts
/// (the connect flow's own state this wave), the routing order view, the
/// limits view, the attribution seam, and the panel's focus bookkeeping.
/// The `ui.rs` seams only read and render it.
pub(crate) struct ProvidersState {
    open: bool,
    /// The connected accounts, in connect order (the fake seam's state —
    /// the real account store wiring lands with the F7 persistence
    /// slice).
    accounts: Vec<ProviderAccountView>,
    /// The account-name entry for the connect form.
    name_input: Entity<InputState>,
    /// The API-key entry for the connect form (masked — the key is never
    /// displayed, and never stored: only the minted reference survives).
    key_input: Entity<InputState>,
    /// The routing order the surface shows and changes (the policy
    /// record's view this wave).
    order: OrderView,
    /// The per-account spend limit, when set (display only).
    account_spend_limit: Option<u32>,
    /// The workspace spend limit, when set (display only).
    workspace_spend_limit: Option<u32>,
    /// The per-account concurrency limit, when set (display only).
    account_concurrency_limit: Option<u32>,
    /// The workspace concurrency limit, when set (display only).
    workspace_concurrency_limit: Option<u32>,
    /// The selected task's attribution view-model (the wiring seam).
    attribution: Option<TaskAttributionView>,
    /// The panel's keyboard focus handle (the 019 request-once shape).
    panel_focus: FocusHandle,
    /// Request-once guard for `panel_focus`.
    panel_focus_requested: bool,
    /// The surface that held focus when the panel opened, restored on
    /// close (the 017 close contract).
    focus_before_panel: Option<FocusHandle>,
}

impl ProvidersState {
    /// Builds the closed, empty providers state (the cold-start honest
    /// empty state: no accounts, free-first order, no limits, no
    /// attribution).
    pub(crate) fn new(window: &mut Window, cx: &mut Context<WorkspaceView>) -> Self {
        Self {
            open: false,
            accounts: Vec::new(),
            name_input: cx
                .new(|cx| InputState::new(window, cx).placeholder(CONNECT_NAME_PLACEHOLDER)),
            key_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .masked(true)
                    .placeholder(CONNECT_KEY_PLACEHOLDER)
            }),
            order: OrderView::FreeFirst,
            account_spend_limit: None,
            workspace_spend_limit: None,
            account_concurrency_limit: None,
            workspace_concurrency_limit: None,
            attribution: None,
            panel_focus: cx.focus_handle(),
            panel_focus_requested: false,
            focus_before_panel: None,
        }
    }

    /// Whether the providers panel is open.
    #[allow(dead_code)] // the render paths check the `open` field directly; the module's tests read it
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// The connected accounts, in connect order.
    #[allow(dead_code)] // read by the render paths and the module's tests
    pub(crate) fn accounts(&self) -> &[ProviderAccountView] {
        &self.accounts
    }

    /// The control's summary line: "None connected yet", or
    /// "2 connected · free accounts first".
    pub(crate) fn entry_summary(&self) -> String {
        if self.accounts.is_empty() {
            return ENTRY_SUMMARY_EMPTY.to_owned();
        }
        ENTRY_SUMMARY_SHAPE
            .replace("{count}", &self.accounts.len().to_string())
            .replace("{order}", self.order.label())
    }

    /// Attaches (or clears) the selected task's attribution view-model —
    /// the wiring seam a later slice drives from the real scheduler's
    /// choices. Clearing removes the task-surface line (never a permanent
    /// line).
    #[allow(dead_code)] // the Wave-later scheduler wiring calls it; the module's tests pin the shapes
    pub(crate) fn set_attribution(&mut self, attribution: Option<TaskAttributionView>) {
        self.attribution = attribution;
    }

    /// The selected task's attribution view-model, when wired.
    #[allow(dead_code)] // the see-row path reads it; the module's tests pin the shapes
    pub(crate) fn attribution(&self) -> Option<&TaskAttributionView> {
        self.attribution.as_ref()
    }

    /// Quietly closes the panel for an F1 navigation action (no focus
    /// restore — the deliberate close paths restore through the 017
    /// contract instead). Returns whether anything changed.
    pub(crate) fn close_for_navigation(&mut self) -> bool {
        let changed = self.open;
        self.open = false;
        self.panel_focus_requested = false;
        self.focus_before_panel = None;
        changed
    }
}

// ---------------------------------------------------------------------------
// The UI actions
// ---------------------------------------------------------------------------

/// Opens (or toggles closed) the provider accounts surface. Called from
/// the task-surface control, both palette rows, and the keyboard chord.
/// The panel lives on the task workspace: navigate there first when
/// needed, so it is reachable from any route.
pub(crate) fn open_providers_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.flauz_providers.open {
        close_providers_surface(workspace, window, cx);
        return;
    }
    if workspace.state.route != MainRoute::Tasks {
        workspace.navigate(MainRoute::Tasks, cx);
    }
    // One panel at a time: opening it closes any open rail/shell panel.
    super::flauz_shell::dismiss_shell_surfaces(workspace, window, cx);
    let state = &mut workspace.flauz_providers;
    if state.focus_before_panel.is_none() {
        state.focus_before_panel = window.focused(cx);
    }
    state.open = true;
    state.panel_focus_requested = true;
    cx.notify();
}

/// Closes the surface and restores focus (the 017 close contract).
pub(crate) fn close_providers_surface(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if !workspace.flauz_providers.open {
        return;
    }
    workspace.flauz_providers.open = false;
    workspace.flauz_providers.panel_focus_requested = false;
    let previous = workspace.flauz_providers.focus_before_panel.take();
    workspace.apply_overlay_close_focus_restore(previous, window, cx);
    cx.notify();
}

/// The "See which account a task uses" path: surfaces the selected
/// task's attribution line (or the honest not-yet guidance), then opens
/// the panel so the accounts and the policy are one glance away.
pub(crate) fn see_which_account_a_task_uses(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    if workspace.state.selected_task_id.is_none() {
        workspace.dispatch_command_status(Some(SEE_NO_CHAT_GUIDANCE), cx);
        return;
    }
    if workspace.flauz_providers.attribution.is_none() {
        workspace.dispatch_command_status(Some(SEE_NO_ATTRIBUTION_GUIDANCE), cx);
    }
    open_providers_surface(workspace, window, cx);
}

/// The connect action: reads the form, runs the fake seam (validate →
/// mint → retain nothing), appends the account view, clears the entries,
/// and keeps the panel open on the success state. Failures surface their
/// truthful lines through the command status — never a silent no-op.
pub(crate) fn connect_from_panel(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let name = workspace
        .flauz_providers
        .name_input
        .read(cx)
        .value()
        .to_string();
    let key = workspace
        .flauz_providers
        .key_input
        .read(cx)
        .value()
        .to_string();
    let sequence = workspace.flauz_providers.accounts.len() as u64 + 1;
    match connect_account_through_fake_seam(&name, &key, sequence) {
        Ok(account) => {
            workspace.flauz_providers.accounts.push(account);
            // The entries are cleared: the key never lingers in the form
            // after the reference is minted.
            workspace
                .flauz_providers
                .name_input
                .update(cx, |input, cx| input.set_value("", window, cx));
            workspace
                .flauz_providers
                .key_input
                .update(cx, |input, cx| input.set_value("", window, cx));
            cx.notify();
        }
        Err(failure) => {
            workspace.dispatch_command_status(Some(failure.user_line()), cx);
        }
    }
}

/// The "Change the order" action (and the depletion moment's "Change
/// this"): flips the routing order and states the consequence plainly
/// through the command status. The panel is open afterwards so the new
/// order is visible.
pub(crate) fn change_routing_order(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    let flipped = workspace.flauz_providers.order.flipped();
    workspace.flauz_providers.order = flipped;
    let consequence = match flipped {
        OrderView::PaidFirst => POLICY_CONSEQUENCE_TO_PAID,
        OrderView::FreeFirst => POLICY_CONSEQUENCE_TO_FREE,
    };
    workspace.dispatch_command_status(Some(consequence), cx);
    if !workspace.flauz_providers.open {
        open_providers_surface(workspace, window, cx);
    } else {
        cx.notify();
    }
}

/// The depletion moment's "Change this" action: the same order-change
/// path (the policy is what moves a task off the used-up free tier).
pub(crate) fn change_depleted_routing(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) {
    change_routing_order(workspace, window, cx);
}

// ---------------------------------------------------------------------------
// The renders
// ---------------------------------------------------------------------------

/// Renders the task-surface control row (the visible, labeled entry —
/// layer 1), the attribution affordance line (layer 2, state-driven),
/// and the open providers panel.
pub(crate) fn render_providers_entry(
    workspace: &mut WorkspaceView,
    window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let open = workspace.flauz_providers.open;
    let summary = workspace.flauz_providers.entry_summary();
    let attribution = workspace.flauz_providers.attribution.clone();
    let mut entry = v_flex().flex_none();
    if workspace.flauz_providers.panel_focus_requested {
        // The 019 request-once shape: the panel claims the keyboard on
        // mount so the scoped Escape binding reaches it.
        workspace.flauz_providers.panel_focus_requested = false;
        workspace.flauz_providers.panel_focus.focus(window);
    }
    entry = entry
        .child(
            h_flex()
                .h(px(32.0))
                .px_5()
                .items_center()
                .gap_2()
                .child(
                    Button::new("flauz-providers-entry")
                        .label(ENTRY_LABEL)
                        .icon(IconName::CircleUser)
                        .tooltip(ENTRY_TOOLTIP)
                        .small()
                        .ghost()
                        .selected(open)
                        .on_click(cx.listener(|this, _, window, cx| {
                            open_providers_surface(this, window, cx);
                        })),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(summary),
                ),
        )
        .child(render_attribution_affordance(attribution.as_ref(), cx));
    if open {
        entry = entry.child(render_providers_panel(workspace, window, cx));
    }
    entry.into_any_element()
}

/// Renders the task-surface attribution affordance (layer 2): the
/// "Using your …" line while quota remains, the depletion moment when
/// the free tier is used up and a paid account exists, and NOTHING when
/// no attribution is wired (never a permanent line, never invented
/// data).
fn render_attribution_affordance(
    attribution: Option<&TaskAttributionView>,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let Some(attribution) = attribution else {
        return div().into_any_element();
    };
    if attribution.is_depleted_moment() {
        // The depletion moment (the work order's exact phrase) with the
        // change action attached.
        return h_flex()
            .px_5()
            .py_1()
            .gap_2()
            .items_center()
            .flex_wrap()
            .child(Icon::new(IconName::TriangleAlert).small())
            .child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .child(DEPLETION_LINE.to_owned()),
            )
            .child(
                Button::new("flauz-providers-depletion-change")
                    .label(DEPLETION_CHANGE_ACTION)
                    .small()
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        change_depleted_routing(this, window, cx);
                    })),
            )
            .into_any_element();
    }
    h_flex()
        .px_5()
        .py_1()
        .gap_2()
        .items_center()
        .child(Icon::new(IconName::CircleCheck).small())
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(attribution.line()),
                )
                // Why this account: whose account and the policy rule's
                // plain-words explanation, visible under the line
                // (attribution names its reason - never "some account").
                .child(
                    div()
                        .text_xs()
                        .line_height(px(16.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{} · {}",
                            attribution.account_label, attribution.why
                        )),
                ),
        )
        .into_any_element()
}

/// Renders the providers panel: the connect form (or the accounts list
/// once accounts exist — the form stays available below it), the
/// routing-policy section (see + change, consequences stated), and the
/// spend/concurrency limits display.
fn render_providers_panel(
    workspace: &mut WorkspaceView,
    _window: &mut Window,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    let state = &workspace.flauz_providers;
    let name_input = state.name_input.clone();
    let key_input = state.key_input.clone();
    let panel_focus = state.panel_focus.clone();
    let accounts = state.accounts.clone();
    let order = state.order;
    let (account_spend, workspace_spend, account_concurrency, workspace_concurrency) = (
        state.account_spend_limit,
        state.workspace_spend_limit,
        state.account_concurrency_limit,
        state.workspace_concurrency_limit,
    );

    let mut panel = v_flex()
        .key_context("FlauzProviders")
        .track_focus(&panel_focus)
        .tab_group()
        .tab_stop(true)
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            close_providers_surface(this, window, cx);
        }))
        .mx_5()
        .my_2()
        .p_4()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        // The panel header (layer 2).
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(PANEL_HEADING),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(PANEL_DESCRIPTION),
                ),
        );

    // The honest empty state (layer 5) or the accounts list (layer 6).
    if accounts.is_empty() {
        panel = panel.child(
            v_flex()
                .gap_1()
                .p_3()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(EMPTY_TITLE),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(EMPTY_BODY),
                ),
        );
    } else {
        let mut list = v_flex().gap_1().child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(cx.theme().muted_foreground)
                .child(ACCOUNTS_LIST_CAPTION),
        );
        for (index, account) in accounts.iter().enumerate() {
            list = list.child(render_account_row(index, account, cx));
        }
        panel = panel.child(list);
    }

    // The connect form (the next step, always available).
    panel = panel
        .child(
            v_flex().gap_1().child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .child(CONNECT_HEADING),
            ),
        )
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(CONNECT_PROVIDER_LABEL),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(CONNECT_NAME_LABEL),
                        )
                        .child(Input::new(&name_input).small().cleanable(true)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(CONNECT_KEY_LABEL),
                        )
                        .child(Input::new(&key_input).small().cleanable(true)),
                )
                .child(
                    Button::new("flauz-providers-connect")
                        .label(CONNECT_BUTTON)
                        .icon(IconName::Plus)
                        .small()
                        .primary()
                        .on_click(cx.listener(|this, _, window, cx| {
                            connect_from_panel(this, window, cx);
                        })),
                )
                .child(
                    div()
                        .text_xs()
                        .line_height(px(16.0))
                        .text_color(cx.theme().muted_foreground)
                        .child(CONNECT_SECURE_NOTE),
                ),
        );

    // The routing-policy section: see + change, consequences stated.
    panel = panel.child(
        v_flex()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .child(POLICY_HEADING),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .flex_wrap()
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(20.0))
                            .child(POLICY_ORDER_SHAPE.replace("{order}", order.label())),
                    )
                    .child(
                        Button::new("flauz-providers-change-order")
                            .label(POLICY_CHANGE_BUTTON)
                            .icon(IconName::ArrowRight)
                            .small()
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                change_routing_order(this, window, cx);
                            })),
                    ),
            ),
    );

    // The spend/concurrency limits display (plain words; the honest
    // no-limits fallback).
    let mut limits = v_flex().gap_1().child(
        div()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(LIMITS_CAPTION),
    );
    let has_limits = account_spend.is_some()
        || workspace_spend.is_some()
        || account_concurrency.is_some()
        || workspace_concurrency.is_some();
    if has_limits {
        for line in [
            account_spend.map(|limit| ACCOUNT_SPEND_SHAPE.replace("{limit}", &limit.to_string())),
            workspace_spend
                .map(|limit| WORKSPACE_SPEND_SHAPE.replace("{limit}", &limit.to_string())),
            account_concurrency
                .map(|limit| ACCOUNT_CONCURRENCY_SHAPE.replace("{limit}", &limit.to_string())),
            workspace_concurrency
                .map(|limit| WORKSPACE_CONCURRENCY_SHAPE.replace("{limit}", &limit.to_string())),
        ]
        .into_iter()
        .flatten()
        {
            limits = limits.child(
                div()
                    .text_sm()
                    .line_height(px(20.0))
                    .text_color(cx.theme().muted_foreground)
                    .child(line),
            );
        }
    } else {
        limits = limits.child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(NO_LIMITS_LINE),
        );
    }
    panel = panel.child(limits);

    panel = panel.child(
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(PROVIDERS_ESCAPE_HINT),
    );
    panel.into_any_element()
}

/// Renders one connected account row: whose account, the provider, the
/// tier chip, the quota state in user words, and the storage line (the
/// reference is never shown).
fn render_account_row(
    index: usize,
    account: &ProviderAccountView,
    cx: &mut Context<WorkspaceView>,
) -> AnyElement {
    v_flex()
        .key_context("FlauzProviderAccount")
        .id(("flauz-provider-account", index))
        .gap_1()
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .flex_wrap()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(account.account_label.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(account.provider_label.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(account.tier.chip_label()),
                ),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(cx.theme().muted_foreground)
                .child(account.quota_line()),
        )
        .child(
            div()
                .text_xs()
                .line_height(px(16.0))
                .text_color(cx.theme().muted_foreground)
                .child(ACCOUNT_SECURE_NOTE),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribution_view(remaining: u32, paid_available: bool) -> TaskAttributionView {
        TaskAttributionView {
            account_label: "Personal account".to_owned(),
            provider_label: "OpenAI".to_owned(),
            tier: TierView::Free,
            quota_remaining: remaining,
            quota_limit: 5,
            window_word: WINDOW_WORD_TODAY,
            why: "Free accounts are tried first, and this one had quota left.".to_owned(),
            paid_account_available: paid_available,
        }
    }

    fn all_user_copy() -> Vec<String> {
        let mut copy: Vec<String> = [
            ENTRY_LABEL,
            ENTRY_TOOLTIP,
            ENTRY_SUMMARY_EMPTY,
            ENTRY_SUMMARY_SHAPE,
            PALETTE_ROW_CONNECT_TITLE,
            PALETTE_ROW_CONNECT_DESCRIPTION,
            PALETTE_ROW_SEE_TITLE,
            PALETTE_ROW_SEE_DESCRIPTION,
            PANEL_HEADING,
            PANEL_DESCRIPTION,
            EMPTY_TITLE,
            EMPTY_BODY,
            CONNECT_HEADING,
            CONNECT_PROVIDER_LABEL,
            CONNECT_NAME_LABEL,
            CONNECT_NAME_PLACEHOLDER,
            CONNECT_KEY_LABEL,
            CONNECT_KEY_PLACEHOLDER,
            CONNECT_BUTTON,
            CONNECT_SECURE_NOTE,
            CONNECT_EMPTY_KEY_FAILURE,
            CONNECT_LOOKS_REAL_FAILURE,
            CONNECT_EMPTY_NAME_FAILURE,
            ACCOUNTS_LIST_CAPTION,
            TIER_FREE_LABEL,
            TIER_PAID_LABEL,
            QUOTA_LINE_SHAPE,
            WINDOW_WORD_TODAY,
            QUOTA_DEPLETED_LINE,
            ACCOUNT_SECURE_NOTE,
            POLICY_HEADING,
            POLICY_ORDER_SHAPE,
            ORDER_FREE_LABEL,
            ORDER_PAID_LABEL,
            POLICY_CHANGE_BUTTON,
            POLICY_CONSEQUENCE_TO_PAID,
            POLICY_CONSEQUENCE_TO_FREE,
            LIMITS_CAPTION,
            ACCOUNT_SPEND_SHAPE,
            WORKSPACE_SPEND_SHAPE,
            ACCOUNT_CONCURRENCY_SHAPE,
            WORKSPACE_CONCURRENCY_SHAPE,
            NO_LIMITS_LINE,
            ATTRIBUTION_LINE_SHAPE,
            DEPLETION_LINE,
            DEPLETION_CHANGE_ACTION,
            SEE_NO_CHAT_GUIDANCE,
            SEE_NO_ATTRIBUTION_GUIDANCE,
            PROVIDERS_ESCAPE_HINT,
        ]
        .iter()
        .map(|text| (*text).to_owned())
        .collect();
        // The generated copy renders through the views too.
        let attribution = attribution_view(3, true);
        copy.push(attribution.line());
        let depleted = attribution_view(0, true);
        copy.push(depleted.line());
        let account = ProviderAccountView {
            account_label: "Personal account".to_owned(),
            provider_label: "OpenAI".to_owned(),
            tier: TierView::Free,
            quota_remaining: 3,
            quota_limit: 5,
            window_word: WINDOW_WORD_TODAY,
            secret_reference: "flausec_practice_0001".to_owned(),
        };
        copy.push(account.quota_line());
        let used_up = ProviderAccountView {
            quota_remaining: 0,
            ..account
        };
        copy.push(used_up.quota_line());
        copy.push(POLICY_ORDER_SHAPE.replace("{order}", ORDER_FREE_LABEL));
        copy.push(POLICY_ORDER_SHAPE.replace("{order}", ORDER_PAID_LABEL));
        copy
    }

    /// The Wave-4 addendum §7 law: money/quota language is plain — never
    /// "flausec_", never "token budget", never an internal type name —
    /// and the work order's exact phrases appear verbatim.
    #[test]
    fn providers_copy_uses_user_language_only() {
        for copy in all_user_copy() {
            let lower = copy.to_lowercase();
            for forbidden in [
                "flausec",
                "token budget",
                "token",
                "conn_",
                "task_",
                "ulid",
                "secretref",
                "provider kind",
                "routing policy",
                "quota ledger",
                "scheduling",
                "scheduler",
                "attribution",
                "api_key",
                "prov-001",
                "seam",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "provider copy must never say {forbidden:?}: {copy:?}"
                );
            }
        }
        // The work order's exact phrases, verbatim.
        assert_eq!(
            DEPLETION_LINE,
            "Your free tier is used up today — next run uses your paid account. Change this"
        );
        assert_eq!(
            CONNECT_SECURE_NOTE,
            "The key is stored securely; Flauz never shows it again."
        );
        assert_eq!(PALETTE_ROW_CONNECT_TITLE, "Connect a provider account");
        assert_eq!(PALETTE_ROW_SEE_TITLE, "See which account a task uses");
        // The attribution line renders in the order's exact style.
        assert_eq!(
            attribution_view(3, true).line(),
            "Using your OpenAI free tier — 3 of 5 runs left today"
        );
    }

    /// The connect flow on the fake seam: practice key in, an opaque
    /// reference out, the account appearing with tier + quota state —
    /// and the entered material retained NOWHERE (the Debug scan is the
    /// storage law made checkable).
    #[test]
    fn the_connect_flow_mints_references_and_retains_nothing() {
        // The truthful failure states first: empty key, empty name, and
        // material that looks like a real credential (nothing stored).
        assert_eq!(
            connect_account_through_fake_seam("Personal account", "", 1),
            Err(ConnectFailure::EmptyKey)
        );
        assert_eq!(
            connect_account_through_fake_seam("", "practice-key-1", 1),
            Err(ConnectFailure::EmptyName)
        );
        for looks_real in [
            "sk-proj-abcdefgh1234",
            "ghp_0123456789abcdef",
            "Bearer abc123",
            "xoxb-012345",
            "password hunter2",
        ] {
            assert_eq!(
                connect_account_through_fake_seam("Personal account", looks_real, 1),
                Err(ConnectFailure::LooksReal),
                "material {looks_real:?} must be refused — nothing was stored"
            );
        }
        assert_eq!(
            ConnectFailure::LooksReal.user_line(),
            CONNECT_LOOKS_REAL_FAILURE
        );

        // The success path: the account appears with tier + quota state,
        // and the minted reference carries the frozen prefix (as data).
        let first = connect_account_through_fake_seam(" Personal account ", "practice-key-1", 1)
            .unwrap_or_else(|failure| panic!("the connect flow must succeed: {failure:?}"));
        assert_eq!(first.account_label, "Personal account");
        assert_eq!(first.provider_label, "OpenAI");
        assert_eq!(first.tier, TierView::Free);
        assert_eq!(first.quota_line(), "5 of 5 runs left today");
        assert!(first.secret_reference.starts_with("flausec_"));

        // Multi-account: a second connect mints a distinct reference.
        let second = connect_account_through_fake_seam("Work account", "practice-key-2", 2)
            .unwrap_or_else(|failure| panic!("the connect flow must succeed: {failure:?}"));
        assert_ne!(first.secret_reference, second.secret_reference);
        assert!(second.secret_reference.starts_with("flausec_"));

        // The material never survives the seam: the account views hold
        // no trace of the entered keys.
        for account in [&first, &second] {
            let debug = format!("{account:?}");
            assert!(!debug.contains("practice-key"));
        }
        // The reference is never a credential shape either.
        for account in [&first, &second] {
            for marker in REAL_KEY_MARKERS {
                assert!(
                    !account.secret_reference.contains(marker),
                    "the minted reference must never resemble credential material"
                );
            }
        }
    }

    /// The attribution affordance states: the "Using your …" line while
    /// quota remains, and the depletion moment — only when a paid account
    /// exists to fall back to (never a false "next run uses your paid
    /// account").
    #[test]
    fn the_attribution_line_and_the_depletion_moment_stay_honest() {
        let healthy = attribution_view(3, true);
        assert!(!healthy.is_depleted_moment());
        assert_eq!(
            healthy.line(),
            "Using your OpenAI free tier — 3 of 5 runs left today"
        );

        // The depletion moment: used up AND a paid account exists.
        let depleted = attribution_view(0, true);
        assert!(depleted.is_depleted_moment());
        assert_eq!(depleted.quota_remaining, 0);

        // Used up but NO paid account exists: the depletion moment's
        // "next run uses your paid account" would be a lie — it is not
        // shown.
        let no_paid = attribution_view(0, false);
        assert!(!no_paid.is_depleted_moment());

        // A paid attribution is never the depletion moment.
        let paid = TaskAttributionView {
            tier: TierView::Paid,
            ..attribution_view(0, true)
        };
        assert!(!paid.is_depleted_moment());
        assert_eq!(
            paid.line(),
            "Using your OpenAI paid account — 0 of 5 runs left today"
        );
    }

    /// The quota words and the policy-change consequences: plain words,
    /// consequences stated.
    #[test]
    fn quota_words_and_policy_consequences_are_plain() {
        let account = ProviderAccountView {
            account_label: "Personal account".to_owned(),
            provider_label: "OpenAI".to_owned(),
            tier: TierView::Free,
            quota_remaining: 4,
            quota_limit: 5,
            window_word: WINDOW_WORD_TODAY,
            secret_reference: "flausec_practice_0001".to_owned(),
        };
        assert_eq!(account.quota_line(), "4 of 5 runs left today");
        assert!(!account.is_depleted());
        let used_up = ProviderAccountView {
            quota_remaining: 0,
            ..account
        };
        assert!(used_up.is_depleted());
        assert_eq!(used_up.quota_line(), QUOTA_DEPLETED_LINE);

        // The order flips between exactly two honest shapes, and both
        // consequences are stated.
        assert_eq!(OrderView::FreeFirst.flipped(), OrderView::PaidFirst);
        assert_eq!(OrderView::PaidFirst.flipped(), OrderView::FreeFirst);
        assert_eq!(ORDER_FREE_LABEL, "free accounts first");
        assert_eq!(ORDER_PAID_LABEL, "paid accounts first");
        assert!(POLICY_CONSEQUENCE_TO_PAID.contains("even when free quota is left"));
        assert!(POLICY_CONSEQUENCE_TO_FREE.contains("tells you before it uses a paid account"));

        // The limits display words and the no-limits fallback.
        assert_eq!(
            ACCOUNT_SPEND_SHAPE.replace("{limit}", "100"),
            "Up to 100 uses per account each day"
        );
        assert_eq!(
            WORKSPACE_SPEND_SHAPE.replace("{limit}", "500"),
            "Up to 500 uses across this workspace each day"
        );
        assert_eq!(
            ACCOUNT_CONCURRENCY_SHAPE.replace("{limit}", "2"),
            "At most 2 tasks at a time per account"
        );
        assert_eq!(
            WORKSPACE_CONCURRENCY_SHAPE.replace("{limit}", "8"),
            "At most 8 tasks at a time in this workspace"
        );
        assert!(NO_LIMITS_LINE.contains("Flauz will tell you"));
    }

    /// Layer 3: the palette rows resolve through natural queries.
    #[test]
    fn palette_queries_resolve_to_the_provider_rows() {
        for query in ["connect", "provider", "account", "api key", "quota", "paid"] {
            let title = PALETTE_ROW_CONNECT_TITLE.to_lowercase();
            let description = PALETTE_ROW_CONNECT_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the connect row"
            );
        }
        for query in ["which account", "uses", "task", "tier", "left"] {
            let title = PALETTE_ROW_SEE_TITLE.to_lowercase();
            let description = PALETTE_ROW_SEE_DESCRIPTION.to_lowercase();
            assert!(
                title.contains(query) || description.contains(query),
                "query {query:?} must resolve to the see row"
            );
        }
    }

    /// The PROV-001 registration seams in `ui.rs` (the house
    /// source-inspection style, the ORCH-003 seam-test precedent): the
    /// module declaration, both palette rows (enum + title + dispatch),
    /// the keyboard chord, the scoped escape, the state field, the
    /// task-surface mount, and the navigation close — each tagged
    /// PROV-001, distinct from every prior wave's seams.
    ///
    /// THE D25 GATE-B LESSON IS LAW: the seam test pins the chord's
    /// on_action LISTENER — the handler that makes the KeyBinding do
    /// something — not just the binding. A binding without a listener
    /// dispatches into the void while the palette row works.
    #[test]
    fn providers_seams_are_registered_in_the_ui_seams() {
        let source = include_str!("../ui.rs");

        // The module declaration seam.
        assert!(source.contains("mod flauz_providers;"));
        assert!(source.contains("use flauz_providers::FlauzProvidersShortcut;"));

        // The palette registration seams: both rows, with their titles
        // and dispatches.
        assert!(source.contains("PaletteCommand::ConnectProviderAccount"));
        assert!(source.contains("PaletteCommand::SeeWhichAccountATaskUses"));
        assert!(source.contains("flauz_providers::PALETTE_ROW_CONNECT_TITLE"));
        assert!(source.contains("flauz_providers::PALETTE_ROW_SEE_TITLE"));
        assert!(source.contains("flauz_providers::open_providers_surface(workspace, window, cx)"));
        assert!(
            source
                .contains("flauz_providers::see_which_account_a_task_uses(workspace, window, cx)")
        );

        // The keyboard chord seam (Ctrl+Alt+Shift+P, the letter family —
        // verified conflict-free: only the unshifted Ctrl+Alt+P exists
        // for chat pinning).
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("FlauzProvidersShortcut, None"),
            "the providers chord must be bound"
        );
        assert!(source.contains("Some(\"Ctrl+Alt+Shift+P\")"));

        // THE D25 GATE-B LESSON: the chord-listener seam. A KeyBinding
        // without an `.on_action` listener dispatches into the void —
        // the chord silently no-ops while the palette row works. The
        // listener must be registered on the workspace root next to the
        // picker/gap/agents/save/recovery chord listeners.
        let listener_form: String = "cx.listener(|this, _: &FlauzProvidersShortcut, window, cx| { \
             flauz_providers::open_providers_surface(this, window, cx); })"
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            normalized.contains(&listener_form),
            "the providers chord must have an on_action listener that calls \
             open_providers_surface (the d25 Gate-B lesson — the seam test pins the \
             listener, not just the binding)"
        );

        // The scoped escape seam (the 019 family shape: one Escape
        // through the panel's own focus context, never a trap).
        assert!(source.contains("Some(\"FlauzProviders\")"));

        // The state-field seam.
        assert!(source.contains("flauz_providers: flauz_providers::ProvidersState"));
        assert!(source.contains("flauz_providers::ProvidersState::new(window, cx)"));

        // The task-surface mount seam (whitespace-insensitive: rustfmt
        // legitimately wraps the mount expression — the shell seam-test
        // precedent).
        assert!(
            normalized.contains("flauz_providers::render_providers_entry("),
            "the providers surface must be mounted on the task surface"
        );

        // The navigation-close seam (quiet: the attribution view-model is
        // state-driven and survives navigation).
        assert!(source.contains("self.flauz_providers.close_for_navigation()"));

        // Every PROV-001 seam is tagged.
        let seam_tags = source.matches("PROV-001").count();
        assert!(
            seam_tags >= 8,
            "each seam is tagged PROV-001 (found {seam_tags})"
        );
    }

    /// The d19/d23 discipline, made checkable: the panel's focus paths
    /// follow the request-once + restore contract, and the scoped Escape
    /// dispatches through the deliberate close path.
    #[test]
    fn providers_focus_never_traps() {
        let source = include_str!("flauz_providers.rs");
        // The request-once shape: focus is requested once and consumed
        // by the render.
        assert!(source.contains("panel_focus_requested = true"));
        assert!(source.contains("panel_focus_requested = false"));
        // The 017 close contract: the previously focused surface is
        // captured and restored on close.
        assert!(source.contains("focus_before_panel = window.focused(cx)"));
        assert!(source.contains("apply_overlay_close_focus_restore"));
        // The scoped Escape dispatches through the deliberate close
        // path, restoring focus — never a trap.
        assert!(source.contains("close_providers_surface(this, window, cx)"));
        // The key entry is masked: the API key is never displayed.
        assert!(source.contains(".masked(true)"));
    }
}
