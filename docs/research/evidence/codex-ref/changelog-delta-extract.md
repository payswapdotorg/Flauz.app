# Official ChatGPT & Codex changelog — curated extract (Worker A, 2026-09-16)

Source: https://learn.chatgpt.com/docs/changelog (retrieved 2026-09-16 via page reader).
Provenance: [docs-derived]. Verbatim entry text from 2026-09-14 back through the
26.715 desktop release (2026-07-23) that immediately precedes the parity baseline
26.721.3996.0 (captured 2026-07-24). iOS entries included where they affect
desktop-parity surfaces (Remote pairing). Whitespace normalized; content unedited.

2026-09-14
 
 
 
 GPT-5.5 retires from ChatGPT, ChatGPT Work, and Codex on October 14 
 
 
 
 
 
 
 
On October 14, 2026, GPT-5.5 will retire from ChatGPT, ChatGPT Work, and Codex
on all plans, including consumer, Business, Enterprise, and Edu plans. This
retirement does not apply to the OpenAI API.
For Codex with ChatGPT sign-in, switch from 
gpt-5.5
 to 
gpt-5.6-sol
(GPT-5.6 Sol) before the retirement date.
Update workspace defaults, saved model settings, managed configurations,
custom agents, scheduled tasks, and scripts that still select 
gpt-5.5
.
See 
GPT-5.5 retirement
 and
workspace model availability
for migration guidance.
 
 
 
 
 
2026-09-11
 
 
 
 Quick chats with Pets and Appshots on Windows 
 26.908
 
 
 
 
 
 
 
 
Chat while you work
Keep ChatGPT close while you work in other apps. In the ChatGPT desktop app on
macOS and Windows, you can now type a quick chat from the floating Pets controls
and press 
Enter
 to send it. Use 
@
 to add context and 
$
 to choose a skill.
Select the bell icon to follow progress in your threads, then select a thread
to open the full conversation.
Choose a companion or show just the controls in 
Settings &gt; Pets
. Press
Option+Space
 on macOS or 
Windows+Alt+P
 on Windows to show the controls
and focus Quick Chat. Pressing the shortcut again keeps Quick Chat open.
You can change 
Show pet
 in 
Settings &gt; Keyboard shortcuts
. To hide the
controls, select 
Hide pet
 in the command menu or 
Settings &gt; Pets
.
On macOS, you can also send an appshot to the floating controls. Keep the main
ChatGPT window in the background and choose 
Automatic
 for 
Appshot
destination
.
Learn more about 
Pets
.
Share an app window on Windows
Appshots
 are now available in the ChatGPT desktop app on
Windows. Press both Alt keys to share the frontmost app window with ChatGPT,
then ask a question or describe what you want to do. An appshot includes a
screenshot and available text from that window. You can customize the shortcut
and choose which chat receives it in settings.
Other improvements and bug fixes
Open files directly from a conversation's 
Sources
 panel, or download
 files that ChatGPT can't preview.
Assign a 
Codex Micro
 key to 
Insert text
 and
 add your own reusable text to the active prompt without sending it.
Reset a pet to its default size in settings. Custom pets also stay in place
when you refresh their artwork.
Keep unfinished comments on response text when you switch chats and return.
ChatGPT dictation now respects your saved 
Main language
 preference.
Keep tab widths and scroll positions more stable as you close browser tabs
in full and split views.
 
 
 
 
 
2026-09-08
 
 
 
 ChatGPT for iOS 
 1.2026.244
 
 
 
 
 
 
 
 
New features
Reference other tasks directly in the composer with 
@
 mentions.
Answer live questions while Codex continues working, without losing your draft.
Explore files with Back navigation, recent files, and remembered reading positions.
Choose a starting branch for new worktrees, or include your current local changes.
Worktree setup can now continue in the background on iOS 26, with progress shown in a Live Activity.
Improvements and bug fixes
Voice now respects your selected model and reasoning effort.
Restored missing delegated-task messages and fixed navigation between linked tasks.
Code reviews now complete more reliably, without duplicate activity or stuck running states.
Fixed crashes and stale queued messages when switching from Queue to Steer.
Improved reconnect reliability and preserved device pairing during temporary service outages.
Resolved duplicate plugin skills in composer suggestions.
 
 
 
 
 
2026-09-05
 
 
 
 Codex MCP server removed 
 
 
 
 
 
 
 
The 
codex mcp-server
 command and standalone 
codex-mcp-server
 binary have
been removed after their deprecation on August 24, 2026. Update integrations
that launch either command before upgrading Codex.
Use the 
Codex app server
 for integrations. The app-server
command is experimental and isn't supported for production workloads.
See the 
migration guidance
 for details. Codex continues to
support 
connecting to external MCP servers
 through 
codex mcp
.
 
 
 
 
 
2026-09-01
 
 
 
 ChatGPT for iOS 
 1.2026.237
 
 
 
 
 
 
 
 
New features
Attachments now work across all connected hosts, including Windows and Linux, and support videos from the Photo Library.
Press and hold the attachment button to attach recent photos.
A new Priority view brings running tasks, unread updates, and tasks awaiting your response to the top of the task list.
Queued prompts now sync with the connected host, remain editable, and send even when the app is in the background.
Long-running tasks now show their live working time.
Task menus now include an option to copy the thread ID.
Improvements and bug fixes
Task list loading and organization are faster and more reliable, with simpler date sections and fewer stalls or disappearing projects.
Reconnects are more reliable, resolving stuck Send states, missing approvals, and stale task updates.
Long responses now stream with fewer visual interruptions.
Side chat messages remain available until you close them, even after the chat can no longer reconnect.
 
 
 
 
 
 
 August 2026 
 
 
 
 
 
 
2026-08-26
 
 
 
 ChatGPT for iOS 
 1.2026.230
 
 
 
 
 
 
 
 
New features
Added task search across titles and conversation content on connected hosts.
Added a compact composer gauge for viewing and adjusting reasoning effort.
Added a full-screen editor for longer prompts.
Added configurable Home Screen shortcuts for ChatGPT, Work, and Codex Remote.
Added optional comments to selected response annotations.
Improvements and bug fixes
Improved loading for long threads, with older history fetched as needed.
Refined task list layout, ordering, and pinning to better match desktop.
Inline visualizations now follow iOS appearance and accent settings.
 
 
 
 
 
2026-08-25
 
 
 
 Browser extensions, site tools, and cloud sign-in 
 
 
 
 
 
 
 
More browsers:
 Use the 
ChatGPT browser extension
 in Microsoft Edge, Brave, Opera, and Vivaldi, as well as Chrome. Set up
 your browser in 
Settings &gt; Computer Use
 in the ChatGPT desktop app.
 All five support tab mentions and browser control; Opera doesn't support
 side chat.
Site tools (WebMCP):
 In the desktop app's built-in browser, ChatGPT Work
 and Codex can use 
tools provided by a website
 to work with
 the page. Use GPT-5.6 Sol or GPT-5.6 Terra and update to the latest desktop
 app. Site tools aren't available with GPT-5.6 Luna or in Enterprise or Edu
 workspaces.
Cloud browser sign-in:
 On eligible plans, ChatGPT Work on the web, iOS,
 and Android can ask you to 
sign in to a supported
 website
 through the cloud
 browser. Enter your details in the sign-in flow, not in the chat. Cloud
 browser sessions stay separate from your local browser. Website sign-in
 isn't available for Enterprise or Edu workspaces.
Availability depends on rollout and workspace settings. Website-access and
action-confirmation requirements still apply.
 
 
 
 
 
2026-08-25
 
 
 
 Trigger scheduled tasks from Gmail, Slack, and GitHub events 
 
 
 
 
 
 
 
ChatGPT scheduled tasks can now run when supported events occur in Gmail, Slack,
or GitHub. Filter Gmail messages by sender or subject, watch selected Slack
channels, or respond to pull request activity such as reviews, comments, commit
updates, and merges.
Event-triggered tasks are available in ChatGPT on the web and mobile for
eligible plans. Connect the relevant app and approve its requested access before
creating a task.
The ChatGPT Slack app must be a member of each watched channel, and the
connected GitHub app must have access to each watched repository.
An event-triggered task can't also use a time-based schedule. When matching
events arrive close together, ChatGPT may combine them in one run.
Open 
Scheduled
 to review pending events or choose 
Run now
.
Learn how to trigger scheduled tasks from app
events
.
 
 
 
 
 
2026-08-24
 
 
 
 Codex MCP server command deprecated 
 
 
 
 
 
 
 
The 
codex mcp-server
 command is now deprecated. Use the 
Codex app
server
 instead.
Update, September 5, 2026:
 The command and standalone 
codex-mcp-server
binary have been removed. See the 
migration guidance
.
 
 
 
 
 
2026-08-20
 
 
 
 Codex and ChatGPT updates 
 
 
 
 
 
 
 
New features
Apple Messages:
 Use the 
Apple Messages plugin
 to read and search Messages chats on your Mac and prepare or send messages. It's available on all plans in the ChatGPT desktop app for macOS. You can use the plugin in ChatGPT Work and Codex. By default, ChatGPT sends messages only after you approve the message and its recipients. See the plugin guide for persistent-approval risks, revocation steps, and the known issue with tasks that disable approval prompts.
Site co-editing:
 Where Site collaboration is available, owners can 
invite active members of the same workspace as editors
. Editors can read the Site's live database data, update the Site, save versions, and publish changes after the owner publishes the Site for the first time. Owners retain control of the audience, settings, analytics, ownership, version restoration, and editor access.
Editable Site URLs:
 Where URL editing is available, owners can 
change an existing Site's ChatGPT-hosted address
 without creating another deployment. The previous address redirects to the new URL. Custom domains are a separate, existing feature and aren't changed by this setting.
Computer History in Europe:
 
Computer History
 is now available in the EEA, Switzerland, and the United Kingdom for ChatGPT Pro, Business, and Enterprise users in the ChatGPT desktop app on macOS. It's off by default and requires Memories. Business and Enterprise administrators must enable access before workspace members can choose to turn it on.
Shared thread snapshots:
 On all Codex plans, 
share a read-only snapshot of a local Codex thread
 from the ChatGPT desktop app for macOS. The snapshot doesn't update when the original thread changes. Personal-account links can be opened by anyone with the link; workspace-account links are limited to members of the originating workspace. Codex redacts known secret patterns, but review the shared content because sensitive content may remain. View or revoke links in 
ChatGPT data controls
, under 
Shared links
.
Unified pinned threads:
 Keep the same 
pinned chats
 across the ChatGPT desktop app and iOS.
 
 
 
 
 
2026-08-19
 
 
 
 GitLab support in Codex cloud (Beta) 
 
 
 
 
 
 
 
GitLab support is available in beta on all ChatGPT plans. Connect a GitLab
project to Codex cloud, create an environment for it, start tasks from issues
or merge requests with 
@codex
, and request one-off or automatic merge request
reviews.
The integration runs in Codex cloud. A managed workspace admin can disable the
connector. GitLab-triggered activity requires permission to configure the
applicable webhook. For GitLab Self-Managed or GitLab Dedicated, a workspace admin must
configure the connection, and webhook activity requires GitLab 19.0 or later.
Codex cannot complete a review when GitLab omits a collapsed or oversize diff.
Learn how to 
use Codex with GitLab
.
 
 
 
 
 
2026-08-18
 
 
 
 ChatGPT for iOS 
 1.2026.223
 
 
 
 
 
 
 
 
New features
Added a setting to open ChatGPT directly in Codex Remote on launch.
Added support for standard MCP forms and editable message approvals.
Linked folders now open directly in the files sheet.
Improvements and bug fixes
Improved the New Thread project picker to reflect the selected host's current projects.
Voice now works directly from existing task composers, connects more reliably, and continues task actions in the background.
Improved diff review stability and performance, especially in large workspaces.
Added a Retry action when task messages fail to load.
Fixed large task responses failing to load.
Fixed tasks disappearing or remaining unavailable after being idle, reconnecting, or returning to the task list.
Improved host pairing reliability and prevented enrollment checks from freezing the app.
Improved response annotations and preserved streamed activity when tasks complete.
Canceling or editing a steering message now prevents delivery.
 
 
 
 
 
2026-08-17
 
 
 
 Public plugin catalog CSV export 
 
 
 
 
 
 
 
Eligible ChatGPT Enterprise workspace owners and admins can download a CSV of
the public plugins visible to their workspace. The export includes plugin, app,
and Chat skill names and descriptions, along with developer, version, date
added in UTC, and OpenAI verification metadata.
Open 
Admin &gt; Plugins
, select 
Public
, and
then select the download icon (
Export CSV
). The export uses a public-catalog
snapshot that can be up to 48 hours old and does not include plugins created for
the workspace.
It isn't available in FedRAMP workspaces.
Learn more about 
plugin controls
.
 
 
 
 
 
2026-08-13
 
 
 
 Computer History 
 
 
 
 
 
 
 
Computer History
 is an opt-in feature
in the ChatGPT desktop app on macOS that turns activity across apps and
websites into memories and a timeline that ChatGPT and Codex can use. Choose
which apps and websites contribute, pause collection, and review or delete
your history at any time.
Computer History is available to ChatGPT Pro, Business, and Enterprise users.
Business and Enterprise administrators must enable access before workspace
members can turn it on. Initial availability excludes the European Economic Area (EEA),
Switzerland, and the United Kingdom.
 
 
 
 
 
2026-08-11
 
 
 
 Linux desktop preview and agent imports 
 
 
 
 
 
 
 
Install the ChatGPT desktop app on Linux
The 
ChatGPT desktop app for Linux
 is available in
preview for supported Ubuntu, Debian, and Fedora desktop distributions on x64
and ARM64 processors. Download the 
.deb
 or 
.rpm
 package for your
distribution, then sign in to work with projects, local files, and Codex.
Import setup and recent work from other agents
The desktop app supports 
Claude Code
, 
Claude Cowork
, and
Cursor
. 
Import instructions, settings, skills, plugins, projects, and
recent work
, then turn on automatic updates in
Settings &gt; Import
 to keep imported work in sync.
Codex CLI can also import supported setup and recent chats from Claude Code
and Cursor with 
/import
.
 
 
 
 
 
2026-08-10
 
 
 
 Introducing Daybreak Blue and Daybreak Red 
 
 
 
 
 
 
 
Daybreak now offers two access tiers for approved defenders: Daybreak Blue and
Daybreak Red. Use them to move from security findings to validated fixes in
explicitly authorized engagements.
Start with Daybreak Blue for most defensive security work. It provides access
to general-purpose models such as GPT-5.6 Sol for vulnerability discovery,
secure code review, detection engineering, incident response, malware analysis,
and patch validation.
Daybreak Red provides separately approved access to purpose-trained models such
as GPT-5.6 Cyber for authorized vulnerability reproduction, exploit validation,
penetration testing, red teaming, and complex system analysis.
Access requires approval through
Trusted Access for Cyber
 and
applies only to the approved identity, workspace or API organization and
project, model, and product surface. Daybreak Red requires separate approval
and provisioning; Daybreak Blue access doesn't grant access to Daybreak Red.
Work in an isolated environment, define your engagement scope, use
least-privilege 
permission profiles
, and configure
Auto-review
 for eligible actions before they
cross the sandbox boundary.
Learn more about 
Cyber Safety
,
Codex credit rates
, and
API token prices
.
 
 
 
 
 
2026-08-07
 
 
 
 ChatGPT for iOS 
 1.2026.209
 
 
 
 
 
 
 
 
Improvements and bug fixes
Improved task reconnection with visible progress while keeping existing tasks usable during connection checks.
New tasks now preserve their first prompt and attachments while starting, preventing empty tasks that could not be reopened.
Improved editing performance for long prompts.
Voice conversations now follow your Background Conversations setting.
Approvals and user-input requests now recover reliably after reconnecting, with cleaner approval details in the transcript.
Fixed follow-up 
/goal
 prompts after completing a goal.
Fixed task-list freezes, disappearing task titles, flashing thinking summaries, and transcript crashes.
 
 
 
 
 
 
 July 2026 
 
 
 
 
 
 
2026-07-31
 
 
 
 GPT-5.4 and GPT-5.4 mini retire from Codex on August 31 
 
 
 
 
 
 
 
On August 31, 2026, GPT-5.4 and GPT-5.4 mini will no longer be available in
Codex for users signed in with ChatGPT. GPT-5.4 and GPT-5.4 mini will remain
available on the OpenAI API and Codex sessions authenticated with an API key.
Switch to their recommended replacements:
Replace 
gpt-5.4
 with 
gpt-5.6-terra
 (GPT-5.6 Terra).
Replace 
gpt-5.4-mini
 with 
gpt-5.6-luna
 (GPT-5.6 Luna).
Before the cutoff, update workspace defaults, saved model settings, managed
configurations, custom agents, and scheduled tasks that use either model.
See 
Codex models
 and
workspace model availability
for details.
 
 
 
 
 
2026-07-31
 
 
 
 Record &amp; Replay expands to the EU, UK, and Switzerland 
 
 
 
 
 
 
 
Record &amp; Replay
 is now available in the
European Union, the United Kingdom, and Switzerland. On macOS, demonstrate a
workflow and turn it into a reusable skill. Computer Use must also be available
and enabled.
 
 
 
 
 
2026-07-30
 
 
 
 Browser upgrades, multi-repository review, and image editing 
 26.727
 
 
 
 
 
 
 
 
The latest ChatGPT desktop app update makes browsing, reviewing code, and
editing generated images faster and easier.
Browse and find context faster
Type in the built-in browser's address bar to revisit pages from your
browsing history or search Google when there's no match.
Manage your browsing history in Settings, and let ChatGPT search that
history when a task needs to find a page you visited before.
Use the Chrome extension to mention open tabs or bring highlighted page
text into your side chat.
Ask questions about any YouTube video in the Chrome extension and get
answers in seconds.
Right-click a webpage and select 
Ask ChatGPT
.
Learn more about the 
built-in browser
 and the
Chrome extension
.
Review changes across repositories
See all repositories in a 
multi-folder project
and the lines changed in each one. Select 
Review
 to inspect diffs across
those repositories without switching between separate review views.
Refine generated images
Open generated images in an expanded viewer, and switch between 
Focused view
and 
Canvas view
. Add comments across images, choose the ones you want, and
send targeted edits without leaving your conversation. Learn more about
image generation
.
Other improvements and bug fixes
Added a new "Activity view" in the sidebar to view which chats you engaged with recently and require attention. Click the bell or use 
Cmd
/
Ctrl
+
Opt
+
U
 to change to the new view.
Updated browser settings to show only supported browsers.
Improved Windows installation reliability when package file paths are long.
Other performance and bug fixes.
 
 
 
 
 
2026-07-29
 
 
 
 Sign in with ChatGPT (beta) 
 
 
 
 
 
 
 
Sign in with ChatGPT is beginning to roll out in beta across select plugins and
partner sites, starting with Airtable, GitLab, HubSpot, Notion, Supabase, and
Vercel.
When you connect a supported plugin from the ChatGPT plugin directory, you can
use Sign in with ChatGPT to create or link an account with that service in fewer
steps. On participating partner sites, you can also select 
Sign in with
ChatGPT
 to create or access your account.
This makes it easier to start working with supported tools in ChatGPT and Codex.
When you sign in, the partner receives only your name, email address, and profile
picture, if available. You must still review and approve each plugin's requested
access as a separate step.
 
 
 
 
 
2026-07-27
 
 
 
 ChatGPT for iOS 
 1.2026.202
 
 
 
 
 
 
 
 
Improvements and bug fixes
Voice conversations now use your selected ChatGPT voice and show usage-limit warnings.
Improved task reconnection and continuity when returning to the app or unlocking with Face ID.
Composer autocomplete now matches desktop plugin mentions and includes skills from installed plugins.
Selected-text references remain available after sending so you can preview them again.
Improved goal controls with clearer progress when pausing or resuming.
Inline visualizations now render tables and visual themes more reliably.
Large workspace diffs are more responsive.
Fixed restored tasks changing the selected model.
Prevented the composer from becoming stuck after a prompt started.
Corrected browser and computer tool labels, icons, and placeholder output.
 
 
 
 
 
2026-07-23
 
 
 
 ChatGPT Voice and multi-folder projects 
 26.715
 
 
 
 
 
 
 
 
Powered by GPT-Live, ChatGPT Voice lets you talk through work and coordinate
tasks in Chat, Work, and Codex in the ChatGPT desktop app.
Start a new chat or task in voice mode, then ask ChatGPT to start, check, or
steer work in other threads. On macOS, turn on 
Screen context
 to share an
appshot
 of your frontmost window.
Voice is available with Plus, Pro, Business, Edu, and Enterprise plans in the
desktop app and through 
Remote on iOS
.
Local projects in the ChatGPT desktop app can now include multiple related
folders. From a project's menu, select 
Edit project
 to add folders and choose
the primary folder. New chats, Git operations, and automatic discovery of
AGENTS.md
, skills, and 
config.toml
 use the primary folder. Secondary
folders remain available for file search, reading, and editing.
Get started with 
ChatGPT Voice
 and 
multi-folder local
projects
.
 
 
 
 
 
