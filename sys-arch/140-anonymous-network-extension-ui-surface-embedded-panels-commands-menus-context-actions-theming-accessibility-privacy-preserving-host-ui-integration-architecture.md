# Core System Architecture Part 140 — Anonymous Network Extension UI Surface, Embedded Panels, Commands, Menus, Context Actions, Theming, Accessibility & Privacy-Preserving Host UI Integration Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 140  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 18, 21–23, 68, 130–139 and UI/UX Parts 01–27

**Primary purpose:** define SIAR's host UI integration architecture for extension-contributed panels, commands, menus, context actions, navigation destinations, theming, accessibility, lifecycle, input mediation, permission-aware rendering, Dioxus desktop integration, Android Compose integration, sandboxed UI messaging, and privacy-preserving extension presentation.

---

# 1. Purpose

Extensions need user-facing surfaces.

They may want to contribute:

```text
settings panels
conversation tools
context-menu actions
toolbar commands
dashboard cards
status views
configuration screens
```

But extension UI creates a new trust boundary.

Without strong design, an extension could:

```text
impersonate core UI
capture sensitive input
mislead users about permissions
read host state through layout side channels
inject arbitrary styles
break accessibility
steal keyboard focus
```

The governing principle is:

> **SIAR extension UI is host-owned presentation with extension-declared intent, not arbitrary UI code inserted into trusted product surfaces.**

---

# 2. Architectural Position

```text
                 EXTENSION PACKAGE
                        │
                        ▼
                 UI CONTRIBUTION
                        │
             ┌──────────┼──────────┐
             │          │          │
          PANELS     COMMANDS     ACTIONS
             │          │          │
             └──────────┼──────────┘
                        ▼
                 HOST UI REGISTRY
                        │
                        ▼
              POLICY / ACCESSIBILITY
                        │
                        ▼
                 PLATFORM ADAPTERS
                DIOXUS / COMPOSE
```

---

# 3. Core Separation

Keep distinct:

```text
extension UI declaration
host UI component
extension runtime state
command/action registration
navigation destination
theme tokens
user input
authorization
```

---

# 4. Non-Goals

Part 140 does not create:

```text
arbitrary DOM injection
extension-owned top-level app chrome
unrestricted CSS
raw host widget handles
core-UI impersonation
```

---

# 5. UI Contribution Identity

```rust
pub struct ExtensionUiContributionId(pub [u8; 16]);
```

---

# 6. Contribution Class

```rust
pub enum ExtensionUiContributionClass {
    Panel,
    SettingsSection,
    Command,
    MenuItem,
    ContextAction,
    StatusCard,
    DetailView,
}
```

---

# 7. Host-Owned Rendering

Hard rule.

---

# 8. Declarative UI Contract

Extensions describe UI intent through a typed schema.

---

# 9. UI Manifest

```rust
pub struct ExtensionUiManifest {
    pub extension: ExtensionId,
    pub contributions: Vec<ExtensionUiContribution>,
}
```

---

# 10. Contribution

```rust
pub struct ExtensionUiContribution {
    pub id: ExtensionUiContributionId,
    pub class: ExtensionUiContributionClass,
    pub location: ExtensionUiLocation,
    pub required_permission: Option<ExtensionPermissionClass>,
}
```

---

# 11. Hard Rule

Location and permission are explicit.

---

# 12. UI Location

```rust
pub enum ExtensionUiLocation {
    Settings,
    ConversationToolbar,
    ConversationContextMenu,
    MessageContextMenu,
    Sidebar,
    Dashboard,
    ExtensionDetail,
}
```

---

# 13. Host Controls Placement

Hard rule.

---

# 14. No Arbitrary Pixel Coordinates

Hard rule.

---

# 15. Embedded Panel

```rust
pub struct ExtensionPanelDescriptor {
    pub contribution: ExtensionUiContributionId,
    pub title: LocalizedStringKey,
    pub view_schema: ExtensionViewSchemaId,
}
```

---

# 16. Panel Surface

Host provides:

```text
layout container
navigation shell
theme tokens
accessibility semantics
input routing
```

---

# 17. Hard Rule

Extension cannot replace host window chrome.

---

# 18. UI Schema

```rust
pub enum ExtensionUiNode {
    Text,
    Button,
    Toggle,
    TextField,
    Select,
    List,
    Card,
    Progress,
    Image,
    Divider,
    Stack,
}
```

---

# 19. Initial Surface Deliberately Small

Hard rule.

---

# 20. No Arbitrary HTML

Hard rule.

---

# 21. No Arbitrary JavaScript

Hard rule.

---

# 22. Native Host Widgets

Host maps schema to native Dioxus/Compose components.

---

# 23. Hard Rule

Semantics stay host-owned.

---

# 24. UI Tree

```rust
pub struct ExtensionViewTree {
    pub root: ExtensionUiNodeId,
    pub nodes: BTreeMap<ExtensionUiNodeId, ExtensionUiNodeDescriptor>,
}
```

---

# 25. Tree Bounds

Bound:

```text
node count
depth
text length
action count
image size
```

---

# 26. Hard Rule

No unbounded UI tree.

---

# 27. Text Content

Localized strings preferred.

---

# 28. Hard Rule

No hidden zero-size text used for covert state.

---

# 29. Image Content

Package resources or brokered user-approved media.

---

# 30. Hard Rule

No remote tracking image URL by default.

---

# 31. Button Action

```rust
pub struct ExtensionUiActionBinding {
    pub node: ExtensionUiNodeId,
    pub action: ExtensionActionId,
}
```

---

# 32. Hard Rule

UI action maps to registered host action, not arbitrary code pointer.

---

# 33. Commands

Commands are first-class.

```rust
pub struct ExtensionCommandDescriptor {
    pub command_id: ExtensionCommandId,
    pub label: LocalizedStringKey,
    pub required_permission: Option<ExtensionPermissionClass>,
    pub contexts: BTreeSet<CommandContext>,
}
```

---

# 34. Command Context

```rust
pub enum CommandContext {
    Global,
    Conversation,
    Message,
    Selection,
    Settings,
}
```

---

# 35. Hard Rule

Command scope cannot exceed active selection/context.

---

# 36. Menu Contribution

```rust
pub struct ExtensionMenuContribution {
    pub menu: HostMenuId,
    pub command: ExtensionCommandId,
    pub placement: MenuPlacementHint,
}
```

---

# 37. Placement Hint

Suggestion only.

---

# 38. Hard Rule

Host decides final ordering.

---

# 39. No Extension Absolute Ordering

Hard rule.

---

# 40. Context Action

```rust
pub struct ExtensionContextAction {
    pub command: ExtensionCommandId,
    pub applies_to: ContextTargetClass,
}
```

---

# 41. Context Target Class

```rust
pub enum ContextTargetClass {
    Conversation,
    Message,
    Attachment,
    Contact,
    File,
}
```

---

# 42. Hard Rule

Context action receives opaque target ref, not raw host object.

---

# 43. Context Snapshot

Host provides minimized projection.

---

# 44. Hard Rule

Selection visibility does not imply full read permission.

---

# 45. Example

Message context menu contribution may know:

```text
a message is selected
```

without receiving message content.

---

# 46. Hard Rule

Action execution rechecks permission.

---

# 47. Toolbar Contribution

Limited count.

---

# 48. Hard Rule

Extensions cannot crowd out core actions.

---

# 49. Overflow

Host may place extra commands in overflow menu.

---

# 50. Hard Rule

No extension controls core toolbar layout.

---

# 51. Sidebar Contribution

Possible for substantial extension surface.

---

# 52. Hard Rule

Sidebar contribution clearly labeled as extension.

---

# 53. Dashboard Card

Host-defined card shape.

---

# 54. Hard Rule

No arbitrary full-screen takeover from dashboard card.

---

# 55. Settings Integration

Extensions get:

```text
extension settings page
permission overview
data controls
background controls
```

---

# 56. Hard Rule

Core security/privacy settings remain separate.

---

# 57. Navigation Destination

```rust
pub struct ExtensionRouteDescriptor {
    pub route_id: ExtensionRouteId,
    pub contribution: ExtensionUiContributionId,
    pub required_permission: Option<ExtensionPermissionClass>,
}
```

---

# 58. Navigation

Host router owns stack/history.

---

# 59. Hard Rule

Extension cannot rewrite global back stack arbitrarily.

---

# 60. Deep-Link Integration

Part 139 route validation applies.

---

# 61. Hard Rule

Deep link enters extension surface through host router.

---

# 62. Foreground Integration

Notification action may open extension panel.

---

# 63. Hard Rule

Opening panel does not broaden runtime authority.

---

# 64. UI Session

```rust
pub struct ExtensionUiSession {
    pub session_id: ExtensionUiSessionId,
    pub extension: ExtensionId,
    pub contribution: ExtensionUiContributionId,
    pub runtime: Option<ExtensionRuntimeId>,
}
```

---

# 65. Hard Rule

UI session is distinct from runtime session.

---

# 66. Runtime Not Running

Host may start runtime after authorization.

---

# 67. Hard Rule

No stale runtime resurrection.

---

# 68. UI State

Host owns transient UI state.

---

# 69. Extension Owns Domain State Through SDK/Runtime

Hard rule.

---

# 70. UI Snapshot

```rust
pub struct ExtensionUiSnapshot {
    pub version: ExtensionUiStateVersion,
    pub tree: ExtensionViewTree,
}
```

---

# 71. Incremental Update

```rust
pub struct ExtensionUiPatch {
    pub base_version: ExtensionUiStateVersion,
    pub operations: Vec<ExtensionUiPatchOp>,
}
```

---

# 72. Hard Rule

Patch size/count bounded.

---

# 73. Patch Validation

Host validates:

```text
tree bounds
node types
action refs
text limits
resource refs
```

---

# 74. Hard Rule

Invalid patch rejected without corrupting current view.

---

# 75. Snapshot Recovery

If patch base mismatch, host requests full snapshot.

---

# 76. Hard Rule

No speculative patching.

---

# 77. UI Communication

Uses Part 137 communication layer.

---

# 78. Hard Rule

No separate ungoverned UI IPC channel.

---

# 79. UI Event

```rust
pub enum ExtensionUiEvent {
    Click(ExtensionUiNodeId),
    ToggleChanged(ExtensionUiNodeId, bool),
    TextChanged(ExtensionUiNodeId, BoundedString),
    SelectionChanged(ExtensionUiNodeId, ExtensionUiValue),
    FocusChanged(ExtensionUiNodeId, bool),
}
```

---

# 80. Hard Rule

Events are bounded and typed.

---

# 81. Sensitive Input

Passwords/secrets use dedicated secure field type.

---

# 82. Hard Rule

Secret input cannot be logged or included in generic telemetry.

---

# 83. Secure Input Node

```rust
pub struct SecureInputDescriptor {
    pub node: ExtensionUiNodeId,
    pub purpose: SecureInputPurpose,
}
```

---

# 84. Secure Input Purpose

```rust
pub enum SecureInputPurpose {
    ApiKey,
    Password,
    Token,
}
```

---

# 85. Hard Rule

Host may store secret directly in secret broker without returning plaintext to extension where feasible.

---

# 86. File Picker

Host-owned.

---

# 87. Hard Rule

Extension gets brokered file handle, not arbitrary path.

---

# 88. Camera/Mic Picker

Host/platform-owned.

---

# 89. Hard Rule

UI button cannot bypass Part 135 permission model.

---

# 90. Confirmation Dialogs

High-risk confirmation is host-owned.

---

# 91. Hard Rule

Extension cannot fake platform permission dialog.

---

# 92. Modal Dialog

Extensions may request generic modal.

---

# 93. Hard Rule

Host labels extension source.

---

# 94. No Full-Screen Blocking Modal Loop

Hard rule.

---

# 95. Focus Management

Host controls focus.

---

# 96. Hard Rule

Extension cannot repeatedly steal focus.

---

# 97. Keyboard Shortcuts

Command can suggest shortcut.

---

# 98. Hard Rule

Host resolves conflicts.

---

# 99. Reserved Shortcuts

Unavailable to extensions.

---

# 100. Hard Rule

No override of security/emergency shortcuts.

---

# 101. Accessibility

Mandatory.

---

# 102. Semantic Roles

```rust
pub enum ExtensionAccessibilityRole {
    Button,
    Checkbox,
    TextField,
    Heading,
    List,
    ListItem,
    Image,
    Status,
}
```

---

# 103. Hard Rule

Interactive node must expose semantic role/name/state.

---

# 104. Screen Readers

Host maps semantics to platform accessibility APIs.

---

# 105. Hard Rule

Extension cannot disable accessibility tree.

---

# 106. Keyboard Navigation

Required for desktop interactive surfaces.

---

# 107. Hard Rule

No mouse-only critical action.

---

# 108. Touch Targets

Host enforces minimum sizes on mobile.

---

# 109. Hard Rule

Extension cannot shrink below accessibility minimum.

---

# 110. Dynamic Type / Font Scaling

Host controls.

---

# 111. Hard Rule

Extension cannot lock absolute font size for core text.

---

# 112. Contrast

Theme token system guarantees accessible defaults.

---

# 113. Hard Rule

Extension cannot submit arbitrary foreground/background color pairs.

---

# 114. Motion

Respect reduced-motion preference.

---

# 115. Hard Rule

No mandatory animation for understanding state.

---

# 116. Theming

Extensions consume semantic tokens.

---

# 117. Theme Token

```rust
pub enum ExtensionThemeToken {
    Surface,
    SurfaceElevated,
    TextPrimary,
    TextSecondary,
    Accent,
    Danger,
    Success,
    Border,
    FocusRing,
}
```

---

# 118. Hard Rule

No raw global CSS/theme override.

---

# 119. Light/Dark/System

Host resolves tokens.

---

# 120. Hard Rule

Extension must work across supported appearances.

---

# 121. Branding Accent

Optional limited token.

---

# 122. Hard Rule

Brand accent cannot reduce accessibility.

---

# 123. Typography

Use host typography roles.

---

# 124. Hard Rule

No arbitrary external font loading.

---

# 125. Iconography

Use:

```text
host icon set
package-local reviewed icon
```

---

# 126. Hard Rule

No remote icon tracking.

---

# 127. Reserved Icons

Security/verified/system icons reserved.

---

# 128. Hard Rule

Extension cannot impersonate core security status.

---

# 129. Layout

Host-provided layout primitives.

---

# 130. Hard Rule

No unrestricted absolute positioning over host UI.

---

# 131. Z-Order

Host-owned.

---

# 132. Hard Rule

Extension cannot overlay invisible click-capture layer.

---

# 133. Input Hit Testing

Native host component tree.

---

# 134. Hard Rule

No transparent clickjacking surface.

---

# 135. Clipboard

UI copy/paste operations still require clipboard policy.

---

# 136. Hard Rule

Text field presence does not grant clipboard read.

---

# 137. Drag And Drop

Optional.

---

# 138. Hard Rule

File/data drop passes through broker and permission checks.

---

# 139. Context Data Projection

Host provides typed context object.

---

# 140. Example

```rust
pub struct MessageContextRef {
    pub conversation: SdkConversationId,
    pub message: SdkMessageId,
    pub content_accessible: bool,
}
```

---

# 141. Hard Rule

Reference itself does not reveal content.

---

# 142. List Virtualization

Host controls large lists.

---

# 143. Hard Rule

Extension cannot demand entire data set loaded at once.

---

# 144. Pagination

SDK query APIs remain bounded.

---

# 145. Hard Rule

UI layer cannot bypass backend pagination.

---

# 146. Performance Budget

Per extension surface.

---

# 147. UI Budget

```rust
pub struct ExtensionUiResourceBudget {
    pub max_nodes: u32,
    pub max_patch_ops_per_second: u32,
    pub max_image_bytes: u64,
    pub max_event_rate: u32,
}
```

---

# 148. Hard Rule

UI flood is throttled.

---

# 149. Rendering Stall

Extension runtime stall should not freeze host UI.

---

# 150. Hard Rule

Host retains last valid snapshot and shows degraded state.

---

# 151. Degraded UI

```rust
pub enum ExtensionUiAvailability {
    Ready,
    Loading,
    Degraded,
    RuntimeUnavailable,
    PermissionRequired,
    UpdateRequired,
}
```

---

# 152. Hard Rule

Core app remains usable if extension UI fails.

---

# 153. Crash Isolation

Extension UI runtime crash does not crash host.

---

# 154. Hard Rule

No native widget pointer sharing.

---

# 155. Dioxus Desktop Adapter

Recommended flow:

```text
Dioxus host component
→ extension UI presenter
→ typed view tree
→ host-native components
→ action router
```

---

# 156. Hard Rule

Extension does not mount arbitrary Dioxus component code into host process unless first-party/trusted architecture explicitly says so.

---

# 157. Android Compose Adapter

Recommended flow:

```text
Compose host screen
→ Rust presentation service / binding
→ typed extension view tree
→ native Compose components
```

---

# 158. Hard Rule

Kotlin rendering does not own extension business authority.

---

# 159. Shared Semantic Model

Desktop and Android share:

```text
node semantics
actions
accessibility labels
theme tokens
```

---

# 160. Hard Rule

Do not force pixel-identical layouts.

---

# 161. Responsive Layout

Host adapts to:

```text
desktop
tablet
foldable
phone
```

---

# 162. Hard Rule

Extension declares intent, not fixed screen coordinates.

---

# 163. Platform-Specific Contribution

Allowed when capability truly platform-specific.

---

# 164. Hard Rule

Unsupported platform clearly omitted.

---

# 165. Settings Surface

Can expose:

```text
extension options
permissions
background work
data storage
notification preferences
```

---

# 166. Hard Rule

Security/privacy controls reflect actual platform state.

---

# 167. No Fake Toggle

Hard rule.

---

# 168. Optimistic UI

Allowed only for reversible low-risk local state.

---

# 169. Hard Rule

Permission/security-sensitive state updates only after authoritative confirmation.

---

# 170. User Consent UX

Part 135 host-owned dialogs.

---

# 171. Hard Rule

Extension UI can request consent but cannot render its own authoritative consent chrome.

---

# 172. Permission Explanation

Extension UI may link to host permission center.

---

# 173. Hard Rule

No duplicate misleading permission UI.

---

# 174. Extension Identity Banner

For substantial embedded panels, host can show:

```text
extension name
publisher
permission indicator
```

---

# 175. Hard Rule

Source identity always recoverable.

---

# 176. Trust Signals

Part 133 factual signals may be shown.

---

# 177. Hard Rule

No one “safe” badge that replaces permission review.

---

# 178. Marketplace Install Preview

UI contribution screenshots optional.

---

# 179. Hard Rule

Screenshots not trusted as runtime behavior evidence.

---

# 180. Extension Update

UI schema compatibility validated before activation.

---

# 181. Hard Rule

Breaking UI contribution changes follow compatibility/lifecycle policy.

---

# 182. Contribution Version

```rust
pub struct ExtensionUiSchemaVersion(pub u32);
```

---

# 183. Hard Rule

Unknown schema version not rendered.

---

# 184. Backward Compatibility

Host may support limited prior schema versions.

---

# 185. Hard Rule

No permissive rendering of unknown privileged node type.

---

# 186. Removed Contribution

Host removes UI entry without leaving stale command/action.

---

# 187. Hard Rule

Command registry and route registry update atomically with contribution set.

---

# 188. Extension Disable

All UI contributions hidden/disabled.

---

# 189. Hard Rule

No orphan menu items.

---

# 190. Extension Revocation

UI surface may remain only as platform-controlled data/export/removal screen.

---

# 191. Hard Rule

Revoked extension code cannot render.

---

# 192. Quarantine UX

Show:

```text
extension disabled
reason category
data/export controls
update/remove actions
```

---

# 193. Hard Rule

Do not expose sensitive internal incident details.

---

# 194. UI Data Access

Every request from view/action goes through normal data brokers.

---

# 195. Hard Rule

Being visible in UI does not widen data access.

---

# 196. Input Data Labels

Sensitive input can be tagged.

---

# 197. Hard Rule

Secret/private input cannot flow to external sink without explicit policy.

---

# 198. UI Telemetry

Allowed:

```text
render errors
node count
patch failures
action error class
```

---

# 199. Forbidden:

```text
full typed text
private field values
per-user clickstream
behavioral funnel
```

---

# 200. Hard Rule

No keystroke telemetry.

---

# 201. No Session Replay

Hard rule.

---

# 202. UI Analytics

Aggregate technical quality only.

---

# 203. Hard Rule

Do not optimize extension consent/attention through dark-pattern analytics.

---

# 204. Localization

Host resolves localized keys.

---

# 205. Hard Rule

Fallback language explicit.

---

# 206. Runtime-Provided Text

Allowed for dynamic content, bounded and sanitized.

---

# 207. Hard Rule

No bidi/control-character spoofing where practical.

---

# 208. Bidirectional Text

Host applies safe Unicode rendering rules.

---

# 209. Hard Rule

Source identity not visually confusable.

---

# 210. Markdown

Optional restricted subset.

---

# 211. Hard Rule

No embedded scripts/images/HTML in extension markdown.

---

# 212. Links

External links clearly indicated.

---

# 213. Hard Rule

Opening external link requires host routing policy.

---

# 214. Form Submission

UI fields generate typed action payload.

---

# 215. Hard Rule

No arbitrary serialized object injection.

---

# 216. Input Schema

```rust
pub struct ExtensionFormSchema {
    pub fields: Vec<ExtensionFormField>,
}
```

---

# 217. Field Type

```rust
pub enum ExtensionFormFieldType {
    Text,
    Number,
    Boolean,
    Select,
    Secret,
    FileHandle,
}
```

---

# 218. Hard Rule

Field values validated before action dispatch.

---

# 219. Form Bounds

Length/range/enumeration constraints explicit.

---

# 220. Hard Rule

No unbounded text/blob input.

---

# 221. Accessibility Test Contract

Every extension UI package with interactive contributions must pass:

```text
semantic role
label
focus order
keyboard path
contrast
text scaling
reduced motion
```

---

# 222. Hard Rule

Accessibility failures can block certification for production marketplace distribution.

---

# 223. Screenshot Testing

Host can render deterministic extension fixture states.

---

# 224. Hard Rule

Screenshots supplement, not replace interaction/accessibility tests.

---

# 225. UI Conformance Testkit

Provides:

```text
fixture host
theme variants
viewport variants
screen reader semantics
keyboard navigation
permission-state variants
```

---

# 226. Hard Rule

Test both authorized and denied states.

---

# 227. Permission-Denied State

Extension view should gracefully show:

```text
permission needed
open host permission center
limited functionality
```

---

# 228. Hard Rule

No endless permission prompt loop.

---

# 229. Offline UI

Show truthful local/offline state.

---

# 230. Hard Rule

No false cloud/sync success.

---

# 231. Background Job UI

Part 138 state can be shown as:

```text
scheduled
waiting
running
failed
paused
```

---

# 232. Hard Rule

Do not expose exact background activity timeline unnecessarily.

---

# 233. Notification UI

Part 139 preferences/actions link into host-owned controls.

---

# 234. Hard Rule

No custom notification-permission screen that misrepresents OS state.

---

# 235. Data Governance UI

Part 136 can expose:

```text
stored size
retention
export
delete
external sharing declaration
```

---

# 236. Hard Rule

Delete/export actions routed through platform services.

---

# 237. Runtime Resource UI

Optional coarse diagnostics:

```text
memory class
background state
recent crash
```

---

# 238. Hard Rule

No host-global resource data.

---

# 239. Developer Diagnostics UI

Only in explicit developer mode.

---

# 240. Hard Rule

No private production content by default.

---

# 241. Extension UI Policy

```rust
pub struct ExtensionUiPolicy {
    pub allowed_locations: BTreeSet<ExtensionUiLocation>,
    pub max_contributions: u32,
    pub resource_budget: ExtensionUiResourceBudget,
    pub allowed_node_types: BTreeSet<ExtensionUiNodeKind>,
}
```

---

# 242. Signed/versioned.

---

# 243. Policy Epoch

```rust
pub struct ExtensionUiPolicyEpoch(pub u64);
```

---

# 244. Hard Rule

Anti-rollback enforced.

---

# 245. Stricter Policy

Can hide/restrict future contributions immediately.

---

# 246. Hard Rule

No fail-open rendering with stale policy.

---

# 247. UI Registry Service

```rust
pub trait ExtensionUiRegistryService {
    fn contributions(
        &self,
        extension: ExtensionId,
    ) -> Result<Vec<ExtensionUiContribution>, ExtensionUiError>;
}
```

---

# 248. View Service

```rust
pub trait ExtensionViewService {
    async fn snapshot(
        &self,
        session: ExtensionUiSessionId,
    ) -> Result<ExtensionUiSnapshot, ExtensionUiError>;
}
```

---

# 249. Command Service

```rust
pub trait ExtensionCommandService {
    async fn invoke(
        &self,
        command: ExtensionCommandId,
        context: CommandInvocationContext,
    ) -> Result<ExtensionActionResult, ExtensionUiError>;
}
```

---

# 250. Navigation Service

```rust
pub trait ExtensionUiNavigationService {
    fn resolve(
        &self,
        route: ExtensionRouteId,
    ) -> Result<ExtensionUiDestination, ExtensionUiError>;
}
```

---

# 251. Error Taxonomy

```rust
pub enum ExtensionUiError {
    ContributionUnknown,
    ContributionNotAllowed,
    UiSchemaUnsupported,
    InvalidViewTree,
    PatchBaseMismatch,
    ResourceBudgetExceeded,
    PermissionDenied,
    RouteInvalid,
    RuntimeUnavailable,
    AccessibilityViolation,
    PolicyStale,
    Unauthorized,
    Internal,
}
```

---

# 252. Observability

Safe metrics:

```text
render success/failure
patch rejection class
node count class
action latency
runtime-unavailable count
```

---

# 253. Forbidden:

```text
typed text
secret values
private clickstreams
user behavior profiles
```

---

# 254. UI SLOs

Examples:

```text
extension panel initial render within target
invalid patch rejection without host crash
command invocation within target
accessibility conformance pass rate
```

---

# 255. Security SLO

```text
0 extension impersonates core security UI
0 arbitrary host widget/pointer access
0 action bypasses permission check
0 transparent clickjacking overlay
```

---

# 256. Privacy SLO

```text
0 secret field values in telemetry
0 remote tracking image fetch by default
0 UI visibility grants broader data access
```

---

# 257. Failure Modes

```text
UI flood
runtime crash
malicious fake consent
theme spoofing
focus abuse
```

---

# 258. UI Flood

Budget/throttle/reject patches.

---

# 259. Runtime Crash

Keep host responsive; show degraded state.

---

# 260. Fake Consent

Platform reserves consent/security surfaces.

---

# 261. Theme Spoofing

Semantic token restrictions.

---

# 262. Focus Abuse

Host-owned focus policy.

---

# 263. Testing

Need extension-UI testkit.

Required scenarios:

```text
settings panel
message context action
theme change
accessibility navigation
runtime unavailable
```

---

# 264. Schema Test

Unsupported UI node/schema rejected.

---

# 265. Bounds Test

Oversized/deep tree rejected.

---

# 266. Permission Test

Hidden/denied contribution cannot invoke protected command.

---

# 267. Context Test

Message action without content permission receives opaque ref only.

---

# 268. Action Revalidation Test

Permission revoked between render and click → action denied.

---

# 269. Accessibility Test

Keyboard/screen reader path reaches every critical action.

---

# 270. Theme Test

Light/dark/high-contrast remain readable.

---

# 271. Focus Test

Extension cannot steal focus repeatedly.

---

# 272. Clickjacking Test

Transparent/invisible overlays rejected by schema/host layout.

---

# 273. Secret Input Test

Secret field omitted from generic UI events/logs.

---

# 274. File Picker Test

Extension gets handle, not arbitrary host path.

---

# 275. Offline Test

UI represents queued/offline state truthfully.

---

# 276. Runtime Crash Test

Host app remains responsive.

---

# 277. Revocation Test

Revoked extension UI removed; platform removal/export screen remains.

---

# 278. Privacy Test

No remote resource loads leak panel visibility.

---

# 279. Fuzzing

Fuzz:

```text
view trees
UI patches
command descriptors
form schemas
deep-link parameters
```

---

# 280. Property Tests

Properties:

```text
UI visibility can never grant a capability
extension contribution can never render outside host-approved location
revoked permission can never authorize a command after click-time revalidation
secret input can never enter generic telemetry pipeline
```

---

# 281. Formal Verification Targets

Strong candidates:

```text
UI session lifecycle
snapshot/patch state
command authorization
contribution registry transitions
```

---

# 282. Kani Candidate

tree bounds/action-permission invariants.

---

# 283. TLA+ Candidate

```text
register → render → interact → revoke/update → remove
```

---

# 284. Loom Candidate

Concurrent:

```text
UI patch
permission revoke
runtime crash
user action
```

---

# 285. Performance

UI integration must not compromise host responsiveness.

---

# 286. Target

```text
bounded patch rate
virtualized lists
async action dispatch
no blocking extension code on UI thread
```

---

# 287. Hard Rule

Extension runtime never executes synchronously on host render thread.

---

# 288. Frame Budget

Host may defer extension updates to protect frame time.

---

# 289. Hard Rule

Core UI responsiveness outranks extension freshness.

---

# 290. Backpressure

UI patch/event channels use Part 137 bounded queues.

---

# 291. Hard Rule

Slow extension does not block input loop.

---

# 292. Cache

Cache only:

```text
validated view snapshot
package-local assets
theme-independent semantic metadata
```

---

# 293. Hard Rule

Permission-sensitive content invalidated when permission changes.

---

# 294. Storage

Separate:

```text
contribution registry
route registry
command registry
validated UI manifests
coarse UI preference state
```

---

# 295. Hard Rule

No UI event behavioral warehouse.

---

# 296. Partitioning

By:

```text
extension
contribution
platform
```

No user-behavior analytics partition.

---

# 297. Crate Layout

Recommended:

```text
crates/
├── siar-extension-ui-core/
├── siar-extension-ui-schema/
├── siar-extension-ui-registry/
├── siar-extension-ui-command/
├── siar-extension-ui-navigation/
├── siar-extension-ui-theme/
├── siar-extension-ui-accessibility/
├── siar-extension-ui-dioxus/
├── siar-extension-ui-compose/
├── siar-extension-ui-observability/
└── siar-extension-ui-testkit/
```

---

# 298. `siar-extension-ui-core`

Owns:

```text
ExtensionUiContributionId
ExtensionUiSessionId
ExtensionUiError
locations
node IDs
```

---

# 299. `siar-extension-ui-schema`

Declarative node/form/patch schemas and validation.

---

# 300. `siar-extension-ui-registry`

Contribution/menu/route registration.

---

# 301. `siar-extension-ui-command`

Typed commands/context actions/action dispatch.

---

# 302. `siar-extension-ui-navigation`

Host-owned extension routes and deep-link integration.

---

# 303. `siar-extension-ui-theme`

Semantic tokens, appearance adaptation, icon/resource rules.

---

# 304. `siar-extension-ui-accessibility`

Semantic roles, focus order, validation.

---

# 305. `siar-extension-ui-dioxus`

Desktop Dioxus host renderer.

---

# 306. `siar-extension-ui-compose`

Android Compose host renderer.

---

# 307. `siar-extension-ui-observability`

Aggregate technical UI health only.

---

# 308. `siar-extension-ui-testkit`

render/action/accessibility/privacy tests.

---

# 309. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension UI is declarative host-rendered presentation; third-party code does not receive arbitrary DOM/native-widget insertion, raw host component pointers, unrestricted CSS, or control of top-level trusted application chrome.
2. UI contribution registration, visibility, rendering, navigation, commands, actions, permissions, runtime authority, and user consent are distinct states; seeing or clicking an extension surface never grants new capability.
3. Every embedded panel, menu item, toolbar command, context action, route, form, and resource reference is schema-validated, bounded, extension-scoped, policy-checked, and removed atomically when the extension is disabled or revoked.
4. Context contributions receive opaque/minimized references and revalidate current authorization at invocation time; selection visibility cannot be used to infer private message/contact/file contents without corresponding permissions.
5. Platform security, permission, consent, verification, identity, and emergency UI are reserved host surfaces that extensions cannot imitate, restyle, replace, overlay, or visually spoof.
6. Theming uses semantic host tokens with accessibility constraints; extensions cannot load arbitrary remote fonts/styles/icons, override security iconography, create transparent clickjacking overlays, or bypass light/dark/high-contrast/reduced-motion preferences.
7. Accessibility is mandatory: interactive contributions expose semantic roles/names/state, keyboard/focus paths, scalable text, sufficient target sizes, and platform accessibility integration; extension UI cannot disable the accessibility tree.
8. Sensitive input is mediated by host-owned secure fields/brokers and is excluded from generic UI events, logs, analytics, screenshots, or telemetry; file/device selection returns brokered handles rather than arbitrary host paths/handles.
9. Extension runtime stalls, crashes, malformed patches, excessive updates, or heavy views cannot freeze the host UI; validated snapshots, bounded patch/event queues, resource budgets, virtualization, and degraded states protect responsiveness.
10. Extension UI telemetry is technical and aggregate—render failures, patch errors, action latency, accessibility failures—and cannot become keystroke logging, session replay, private clickstreams, behavioral profiling, consent optimization, or per-user engagement scoring.
11. Dioxus desktop and Android Compose adapters share semantic presentation contracts while remaining native to each platform; UI adapters never own extension business authority and cannot bypass runtime, permission, data, notification, or background-policy layers.
12. Extension UI integration composes with SDKs, runtime sandboxing, permissions, data governance, IPC, background jobs, notifications, deep links, marketplace trust, compatibility, lifecycle, assurance, and accessibility without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 310. Initial Production Scope

Implement first:

```text
typed contribution registry
declarative bounded UI schema
settings panels
detail panels
commands
menu/context actions
host-owned route registry
opaque context references
snapshot + patch model
permission-aware rendering
host action revalidation
semantic theme tokens
light/dark adaptation
accessibility semantics
secure input fields
brokered file selection
Dioxus desktop renderer
Android Compose renderer
UI resource budgets
degraded/runtime-unavailable states
revocation/update cleanup
privacy-safe UI observability
extension-UI testkit
```

Then add:

```text
dashboard cards
sidebar contributions
drag/drop mediation
advanced adaptive/foldable layouts
extension-local navigation subtrees
richer controlled visualization widgets
formal snapshot/patch verification
automated accessibility certification
```

---

# 311. Definition of Done

Part 140 is complete when:

- extension UI is declarative and host-rendered;
- contribution locations are explicit and bounded;
- commands/context actions are typed and reauthorized at click time;
- context refs do not imply data read access;
- core security/consent UI cannot be impersonated;
- theme tokens replace unrestricted styling;
- accessibility is mandatory;
- secret/file/device input is brokered;
- malformed/slow extension UI cannot freeze the host;
- Dioxus and Compose adapters share semantics without sharing widget code;
- revoked extensions cannot continue rendering;
- no UI telemetry becomes keystroke/clickstream/session surveillance;
- UI/action/accessibility/privacy/fuzz/formal tests are specified.

---

# 312. Final Architecture

```text
                 EXTENSION PACKAGE
                        │
                        ▼
               DECLARATIVE UI MANIFEST
                        │
             ┌──────────┼──────────┐
             │          │          │
          PANELS     COMMANDS    ACTIONS
             │          │          │
             └──────────┼──────────┘
                        ▼
                 HOST UI REGISTRY
                        │
                        ▼
             POLICY / THEME / A11Y
                        │
             ┌──────────┼──────────┐
             │                     │
           DIOXUS                COMPOSE
             │                     │
             └──────────┬──────────┘
                        ▼
                USER INTERACTION
                        │
                        ▼
              HOST ACTION ROUTER
```

Extension-UI safety model:

```text
declarative host rendering
+
typed contributions
+
opaque context refs
+
click-time authorization
+
semantic theming
+
mandatory accessibility
+
brokered sensitive input
+
bounded UI resources
+
privacy-safe telemetry
```

not:

```text
let plugins inject arbitrary UI code, fake permission dialogs, steal focus, read host state through widget handles, and capture every click or keystroke
```

---

# 313. Final Principle

Extension UI is trustworthy when the host remains in control of **placement, identity, semantics, permissions, accessibility, navigation, input, styling, and lifecycle**, while the extension contributes only bounded product intent.

The correct model is:

```text
declare UI intent
+
render with host components
+
scope commands to context
+
revalidate actions
+
use semantic theme tokens
+
enforce accessibility
+
broker sensitive input/resources
+
degrade safely when runtime fails
+
never let presentation become authority
```

This architecture gives SIAR a privacy-preserving host UI integration foundation for embedded panels, menus, commands, context actions, theming, accessibility, navigation, Dioxus desktop rendering, Android Compose rendering, and extension interaction while preserving the anonymity, local-first, least-authority, runtime, permission, data-governance, background, notification, and anti-surveillance guarantees established across Parts 34–139.
